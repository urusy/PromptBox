import asyncio
import shutil
import threading
from pathlib import Path
from typing import ClassVar
from uuid import UUID

from loguru import logger
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from uuid_utils import uuid7
from watchdog.events import FileCreatedEvent, FileSystemEventHandler
from watchdog.observers import Observer

from app.config import get_settings
from app.models.image import Image
from app.parsers import MetadataParserFactory, read_image_info
from app.utils.file_utils import ensure_directory, get_storage_path, get_thumbnail_path
from app.utils.hash_utils import calculate_file_hash
from app.utils.image_utils import create_thumbnail_async, get_image_dimensions_async

# Interval for periodic folder scanning (in seconds)
PERIODIC_SCAN_INTERVAL = 30


class ThreadSafeSet:
    """A thread-safe set implementation using a lock."""

    def __init__(self) -> None:
        self._set: set[str] = set()
        self._lock = threading.Lock()

    def add(self, item: str) -> bool:
        """Add an item to the set. Returns True if added, False if already present."""
        with self._lock:
            if item in self._set:
                return False
            self._set.add(item)
            return True

    def discard(self, item: str) -> None:
        """Remove an item from the set if present."""
        with self._lock:
            self._set.discard(item)

    def __contains__(self, item: str) -> bool:
        """Check if an item is in the set."""
        with self._lock:
            return item in self._set


# Global session factory for the watcher (created once per event loop)
_watcher_session_cache: dict[int, async_sessionmaker[AsyncSession]] = {}
_watcher_session_lock = threading.Lock()


def get_watcher_session() -> async_sessionmaker[AsyncSession]:
    """Get or create a session factory for the current event loop.

    This caches the session factory per event loop to avoid creating
    a new engine for every file processed.
    """
    loop_id = id(asyncio.get_event_loop())

    with _watcher_session_lock:
        if loop_id not in _watcher_session_cache:
            settings = get_settings()
            engine = create_async_engine(
                settings.database_url,
                echo=False,
                pool_size=5,
                max_overflow=10,
                pool_pre_ping=True,  # Check connection health
            )
            _watcher_session_cache[loop_id] = async_sessionmaker(
                engine,
                class_=AsyncSession,
                expire_on_commit=False,
            )
            logger.debug(f"Created new session factory for event loop {loop_id}")

        return _watcher_session_cache[loop_id]


class ImageImportHandler(FileSystemEventHandler):
    """Handler for file system events in the import folder."""

    SUPPORTED_EXTENSIONS: ClassVar[set[str]] = {".png", ".jpg", ".jpeg", ".webp"}

    def __init__(self) -> None:
        self.settings = get_settings()
        self.parser_factory = MetadataParserFactory()
        self._processing = ThreadSafeSet()

    def on_created(self, event: FileCreatedEvent) -> None:
        """Handle file creation events."""
        if event.is_directory:
            return

        file_path = Path(event.src_path)

        # Check extension
        if file_path.suffix.lower() not in self.SUPPORTED_EXTENSIONS:
            return

        # Avoid duplicate processing (atomic check-and-add)
        if not self._processing.add(str(file_path)):
            return

        # Process in a new event loop (watchdog runs in a separate thread)
        try:
            asyncio.run(self._process_file(file_path))
        except OSError as e:
            logger.error(f"I/O error processing {file_path}: {e}")
        except RuntimeError as e:
            logger.error(f"Runtime error processing {file_path}: {e}")
        finally:
            self._processing.discard(str(file_path))

    async def _process_file(self, file_path: Path) -> None:
        """Process a newly created image file."""
        logger.info(f"Processing new file: {file_path}")

        # Wait for file to be fully written
        await self._wait_for_file(file_path)

        if not file_path.exists():
            logger.warning(f"File no longer exists: {file_path}")
            return

        # Get or create session factory for this event loop
        session_factory = get_watcher_session()
        async with session_factory() as db:
            try:
                await self._import_image(db, file_path)
                await db.commit()
            except Exception as e:
                await db.rollback()
                logger.error(f"Failed to import {file_path}: {e}")
                raise

    async def _wait_for_file(self, file_path: Path, timeout: float = 30.0) -> None:
        """Wait for a file to be fully written."""
        start_time = asyncio.get_event_loop().time()
        last_size = -1

        while True:
            if not file_path.exists():
                break

            current_size = file_path.stat().st_size

            # File size hasn't changed for a bit, assume it's done
            if current_size == last_size and current_size > 0:
                await asyncio.sleep(0.5)  # Extra wait
                break

            last_size = current_size

            if asyncio.get_event_loop().time() - start_time > timeout:
                break

            await asyncio.sleep(0.5)

    async def _import_image(self, db: AsyncSession, file_path: Path) -> UUID:
        """Import an image file into the database and storage."""
        # Calculate file hash for duplicate detection
        file_hash = calculate_file_hash(file_path)

        # Check for duplicates
        result = await db.execute(select(Image).where(Image.file_hash == file_hash))
        existing = result.scalar_one_or_none()

        if existing:
            logger.info(f"Duplicate detected: {file_path} (hash: {file_hash[:16]}...)")
            # Move the duplicate file to duplicated folder
            duplicated_dir = Path(self.settings.import_path) / "duplicated"
            ensure_directory(duplicated_dir)
            duplicated_path = duplicated_dir / file_path.name
            # Handle filename collision in duplicated folder
            if duplicated_path.exists():
                stem = file_path.stem
                suffix = file_path.suffix
                counter = 1
                while duplicated_path.exists():
                    duplicated_path = duplicated_dir / f"{stem}_{counter}{suffix}"
                    counter += 1
            shutil.move(str(file_path), str(duplicated_path))
            logger.info(f"Moved duplicate to: {duplicated_path}")
            return existing.id

        # Generate UUID v7 for database ID
        image_id = uuid7()

        # Get image dimensions (async)
        width, height = await get_image_dimensions_async(file_path)

        # Get file size
        file_size = file_path.stat().st_size

        # Parse metadata (supports PNG and JPEG)
        image_info = read_image_info(file_path)
        metadata = self.parser_factory.parse(image_info)

        # Generate paths using file hash (content-addressable storage)
        storage_rel_path = get_storage_path(
            self.settings.storage_path,
            file_hash,
            file_path.name,
        )
        thumbnail_rel_path = get_thumbnail_path(
            self.settings.storage_path,
            file_hash,
        )

        storage_abs_path = Path(self.settings.storage_path) / storage_rel_path
        thumbnail_abs_path = Path(self.settings.storage_path) / thumbnail_rel_path

        # Ensure directories exist
        ensure_directory(storage_abs_path.parent)
        ensure_directory(thumbnail_abs_path.parent)

        # Move file to storage
        shutil.move(str(file_path), str(storage_abs_path))

        # Create thumbnail (async)
        await create_thumbnail_async(
            storage_abs_path,
            thumbnail_abs_path,
            size=self.settings.thumbnail_size,
            quality=self.settings.thumbnail_quality,
        )

        # Create database record
        image = Image(
            id=image_id,
            source_tool=metadata.source_tool.value,
            model_type=metadata.model_type.value if metadata.model_type else None,
            has_metadata=metadata.has_metadata,
            original_filename=file_path.name,
            storage_path=storage_rel_path,
            thumbnail_path=thumbnail_rel_path,
            file_hash=file_hash,
            width=width,
            height=height,
            file_size_bytes=file_size,
            positive_prompt=metadata.positive_prompt,
            negative_prompt=metadata.negative_prompt,
            model_name=metadata.model_name,
            sampler_name=metadata.sampler_name,
            scheduler=metadata.scheduler,
            steps=metadata.steps,
            cfg_scale=metadata.cfg_scale,
            seed=metadata.seed,
            loras=[lora.to_dict() for lora in metadata.loras],
            controlnets=[cn.to_dict() for cn in metadata.controlnets],
            embeddings=[emb.to_dict() for emb in metadata.embeddings],
            model_params=metadata.model_params,
            workflow_extras=metadata.workflow_extras,
            raw_metadata=metadata.raw_metadata,
        )

        db.add(image)
        logger.info(f"Imported: {file_path.name} -> {image_id}")

        return image_id


class ImageWatcher:
    """Watches the import folder for new images."""

    def __init__(self) -> None:
        self.settings = get_settings()
        self.observer = Observer()
        self.handler = ImageImportHandler()
        self._scanner_thread: threading.Thread | None = None
        self._stop_scanner = threading.Event()

    def start(self) -> None:
        """Start watching the import folder."""
        import_path = self.settings.import_path
        ensure_directory(import_path)

        self.observer.schedule(self.handler, import_path, recursive=False)
        self.observer.start()
        logger.info(f"Started watching: {import_path}")

        # Start periodic scanner thread
        self._stop_scanner.clear()
        self._scanner_thread = threading.Thread(
            target=self._periodic_scan_loop,
            daemon=True,
            name="PeriodicScanner",
        )
        self._scanner_thread.start()
        logger.info(f"Started periodic scanner (interval: {PERIODIC_SCAN_INTERVAL}s)")

    def stop(self) -> None:
        """Stop watching."""
        # Stop periodic scanner
        self._stop_scanner.set()
        if self._scanner_thread and self._scanner_thread.is_alive():
            self._scanner_thread.join(timeout=5)
            logger.info("Stopped periodic scanner")

        self.observer.stop()
        self.observer.join()
        logger.info("Stopped watching")

    def _periodic_scan_loop(self) -> None:
        """Background thread that periodically scans for unprocessed files."""
        while not self._stop_scanner.is_set():
            # Wait for the interval (or until stop is signaled)
            if self._stop_scanner.wait(timeout=PERIODIC_SCAN_INTERVAL):
                break  # Stop was signaled

            # Scan for unprocessed files
            try:
                self._scan_and_process()
            except Exception as e:
                logger.error(f"Error in periodic scan: {e}")

    def _scan_and_process(self) -> None:
        """Scan import folder and process any unprocessed files."""
        import_path = Path(self.settings.import_path)

        if not import_path.exists():
            return

        unprocessed_files = []
        for file_path in import_path.iterdir():
            if not file_path.is_file():
                continue
            ext = file_path.suffix.lower()
            # Skip unsupported extensions or files currently being processed
            if (
                ext in ImageImportHandler.SUPPORTED_EXTENSIONS
                and str(file_path) not in self.handler._processing
            ):
                unprocessed_files.append(file_path)

        if unprocessed_files:
            logger.info(
                f"Periodic scan found {len(unprocessed_files)} unprocessed file(s)"
            )
            for file_path in unprocessed_files:
                # Atomic check-and-add to avoid duplicate processing
                if not self.handler._processing.add(str(file_path)):
                    continue
                try:
                    asyncio.run(self.handler._process_file(file_path))
                except OSError as e:
                    logger.error(
                        f"I/O error processing {file_path} in periodic scan: {e}"
                    )
                except RuntimeError as e:
                    logger.error(
                        f"Runtime error processing {file_path} in periodic scan: {e}"
                    )
                finally:
                    self.handler._processing.discard(str(file_path))

    def process_existing(self) -> None:
        """Process any existing files in the import folder."""
        import_path = Path(self.settings.import_path)

        for file_path in import_path.iterdir():
            if file_path.is_file():
                ext = file_path.suffix.lower()
                if ext in ImageImportHandler.SUPPORTED_EXTENSIONS:
                    logger.info(f"Processing existing file: {file_path}")
                    try:
                        asyncio.run(self.handler._process_file(file_path))
                    except OSError as e:
                        logger.error(f"I/O error processing {file_path}: {e}")
                    except RuntimeError as e:
                        logger.error(f"Runtime error processing {file_path}: {e}")
