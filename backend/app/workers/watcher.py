import asyncio
import shutil
import threading
from datetime import datetime, timezone
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


class ImageImportHandler(FileSystemEventHandler):
    """Handler for file system events in the import folder.

    Dispatches file-created events to the worker event loop owned by
    ``ImageWatcher`` (set via ``set_dispatch``). We never create a new event
    loop per file — all processing happens on the single long-lived loop.
    """

    SUPPORTED_EXTENSIONS: ClassVar[set[str]] = {".png", ".jpg", ".jpeg", ".webp"}

    def __init__(self) -> None:
        self.settings = get_settings()
        self.parser_factory = MetadataParserFactory()
        self._processing = ThreadSafeSet()
        self._dispatch: "ImageWatcher._Dispatch | None" = None

    def set_dispatch(self, dispatch: "ImageWatcher._Dispatch") -> None:
        """Bind the dispatcher used to submit coroutines to the worker loop."""
        self._dispatch = dispatch

    def on_created(self, event: FileCreatedEvent) -> None:
        """Handle file creation events (runs in watchdog's thread)."""
        if event.is_directory:
            return

        file_path = Path(event.src_path)

        if file_path.suffix.lower() not in self.SUPPORTED_EXTENSIONS:
            return

        # Avoid duplicate processing (atomic check-and-add)
        if not self._processing.add(str(file_path)):
            return

        if self._dispatch is None:
            # Shouldn't happen — dispatcher is set before observer starts.
            logger.error("Watcher dispatch not initialized; dropping event")
            self._processing.discard(str(file_path))
            return

        async def _run() -> None:
            try:
                await self._process_file(file_path)
            except OSError as e:
                logger.error(f"I/O error processing {file_path}: {e}")
            except Exception as e:  # noqa: BLE001
                logger.error(f"Error processing {file_path}: {e}")
            finally:
                self._processing.discard(str(file_path))

        self._dispatch.submit(_run())

    async def _process_file(self, file_path: Path) -> None:
        """Process a newly created image file."""
        logger.info(f"Processing new file: {file_path}")

        await self._wait_for_file(file_path)

        if not file_path.exists():
            logger.warning(f"File no longer exists: {file_path}")
            return

        assert self._dispatch is not None
        session_factory = self._dispatch.session_factory()
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

            if current_size == last_size and current_size > 0:
                await asyncio.sleep(0.5)
                break

            last_size = current_size

            if asyncio.get_event_loop().time() - start_time > timeout:
                break

            await asyncio.sleep(0.5)

    async def _import_image(self, db: AsyncSession, file_path: Path) -> UUID:
        """Import an image file into the database and storage."""
        file_hash = calculate_file_hash(file_path)

        result = await db.execute(select(Image).where(Image.file_hash == file_hash))
        existing = result.scalar_one_or_none()

        if existing:
            logger.info(f"Duplicate detected: {file_path} (hash: {file_hash[:16]}...)")
            duplicated_dir = Path(self.settings.import_path) / "duplicated"
            ensure_directory(duplicated_dir)
            duplicated_path = duplicated_dir / file_path.name
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

        image_id = uuid7()

        width, height = await get_image_dimensions_async(file_path)

        file_stat = file_path.stat()
        file_size = file_stat.st_size

        try:
            file_timestamp = file_stat.st_birthtime
        except AttributeError:
            file_timestamp = file_stat.st_mtime
        file_created_at = datetime.fromtimestamp(file_timestamp, tz=timezone.utc)

        image_info = read_image_info(file_path)
        metadata = self.parser_factory.parse(image_info)

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

        ensure_directory(storage_abs_path.parent)
        ensure_directory(thumbnail_abs_path.parent)

        shutil.move(str(file_path), str(storage_abs_path))

        await create_thumbnail_async(
            storage_abs_path,
            thumbnail_abs_path,
            size=self.settings.thumbnail_size,
            quality=self.settings.thumbnail_quality,
        )

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
            created_at=file_created_at,
            updated_at=file_created_at,
        )

        db.add(image)
        logger.info(f"Imported: {file_path.name} -> {image_id}")

        return image_id


class ImageWatcher:
    """Watches the import folder for new images.

    Runs a single long-lived asyncio event loop in a dedicated thread; all
    file-processing work is dispatched onto it. This avoids creating a fresh
    event loop per file (previous design) and lets the DB connection pool be
    reused across events.
    """

    class _Dispatch:
        """Glue between threads (watchdog, periodic scanner) and the worker loop."""

        def __init__(self) -> None:
            self._loop: asyncio.AbstractEventLoop | None = None
            self._engine = None
            self._session_factory: async_sessionmaker[AsyncSession] | None = None

        def bind(
            self,
            loop: asyncio.AbstractEventLoop,
            session_factory: async_sessionmaker[AsyncSession],
            engine,
        ) -> None:
            self._loop = loop
            self._session_factory = session_factory
            self._engine = engine

        @property
        def loop(self) -> asyncio.AbstractEventLoop:
            assert self._loop is not None, "Dispatch not bound"
            return self._loop

        def session_factory(self) -> async_sessionmaker[AsyncSession]:
            assert self._session_factory is not None, "Dispatch not bound"
            return self._session_factory

        def submit(self, coro) -> "asyncio.Future":
            """Schedule a coroutine on the worker loop from another thread."""
            return asyncio.run_coroutine_threadsafe(coro, self.loop)

        async def dispose(self) -> None:
            if self._engine is not None:
                await self._engine.dispose()

    def __init__(self) -> None:
        self.settings = get_settings()
        self.observer = Observer()
        self.handler = ImageImportHandler()
        self._dispatch = ImageWatcher._Dispatch()
        self.handler.set_dispatch(self._dispatch)
        self._scanner_thread: threading.Thread | None = None
        self._stop_scanner = threading.Event()
        self._loop_thread: threading.Thread | None = None
        self._loop_ready = threading.Event()

    def _run_loop(self) -> None:
        """Run a persistent asyncio loop in this thread."""
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)

        engine = create_async_engine(
            self.settings.database_url,
            echo=False,
            pool_size=3,
            max_overflow=2,
            pool_pre_ping=True,
            pool_recycle=300,
        )
        session_factory = async_sessionmaker(
            engine, class_=AsyncSession, expire_on_commit=False
        )
        self._dispatch.bind(loop, session_factory, engine)
        self._loop_ready.set()

        try:
            loop.run_forever()
        finally:
            try:
                loop.run_until_complete(self._dispatch.dispose())
            except Exception as e:  # noqa: BLE001
                logger.error(f"Error disposing watcher engine: {e}")
            loop.close()

    def start(self) -> None:
        """Start the worker loop, observer, and periodic scanner."""
        import_path = self.settings.import_path
        ensure_directory(import_path)

        # Start the persistent worker loop (idempotent — process_existing may have
        # already spun it up).
        if self._loop_thread is None or not self._loop_thread.is_alive():
            self._loop_thread = threading.Thread(
                target=self._run_loop, daemon=True, name="WatcherLoop"
            )
            self._loop_thread.start()
            if not self._loop_ready.wait(timeout=5):
                raise RuntimeError("Watcher event loop failed to start")

        self.observer.schedule(self.handler, import_path, recursive=False)
        self.observer.start()
        logger.info(f"Started watching: {import_path}")

        self._stop_scanner.clear()
        self._scanner_thread = threading.Thread(
            target=self._periodic_scan_loop,
            daemon=True,
            name="PeriodicScanner",
        )
        self._scanner_thread.start()
        logger.info(f"Started periodic scanner (interval: {PERIODIC_SCAN_INTERVAL}s)")

    def stop(self) -> None:
        """Stop observer, scanner, and worker loop."""
        self._stop_scanner.set()
        if self._scanner_thread and self._scanner_thread.is_alive():
            self._scanner_thread.join(timeout=5)
            logger.info("Stopped periodic scanner")

        self.observer.stop()
        self.observer.join()
        logger.info("Stopped watching")

        # Stop the persistent worker loop
        if self._loop_thread and self._loop_thread.is_alive():
            loop = self._dispatch.loop
            loop.call_soon_threadsafe(loop.stop)
            self._loop_thread.join(timeout=10)
            logger.info("Stopped watcher event loop")

    def _periodic_scan_loop(self) -> None:
        """Background thread: periodically submit unprocessed files to worker loop."""
        while not self._stop_scanner.is_set():
            if self._stop_scanner.wait(timeout=PERIODIC_SCAN_INTERVAL):
                break

            try:
                self._scan_and_process()
            except Exception as e:  # noqa: BLE001
                logger.error(f"Error in periodic scan: {e}")

    def _scan_and_process(self) -> None:
        """Scan import folder and submit unprocessed files to worker loop."""
        import_path = Path(self.settings.import_path)

        if not import_path.exists():
            return

        unprocessed_files: list[Path] = []
        for file_path in import_path.iterdir():
            if not file_path.is_file():
                continue
            ext = file_path.suffix.lower()
            if (
                ext in ImageImportHandler.SUPPORTED_EXTENSIONS
                and str(file_path) not in self.handler._processing
            ):
                unprocessed_files.append(file_path)

        if unprocessed_files:
            logger.info(
                f"Periodic scan found {len(unprocessed_files)} unprocessed file(s)"
            )
            # Dispatch the batch to the worker loop; wait for completion so
            # we don't overlap with the next scan interval.
            future = self._dispatch.submit(self._process_files_batch(unprocessed_files))
            try:
                future.result(timeout=300)
            except Exception as e:  # noqa: BLE001
                logger.error(f"Periodic scan batch failed: {e}")

    async def _process_files_batch(self, files: list[Path]) -> None:
        """Process multiple files sequentially on the worker loop."""
        for file_path in files:
            if not self.handler._processing.add(str(file_path)):
                continue
            try:
                await self.handler._process_file(file_path)
            except OSError as e:
                logger.error(
                    f"I/O error processing {file_path} in periodic scan: {e}"
                )
            except Exception as e:  # noqa: BLE001
                logger.error(
                    f"Error processing {file_path} in periodic scan: {e}"
                )
            finally:
                self.handler._processing.discard(str(file_path))

    async def process_existing(self) -> None:
        """Process any existing files in the import folder.

        Called from the FastAPI lifespan (which has its own loop). The actual
        processing runs on the worker loop — we just submit and wait.
        """
        import_path = Path(self.settings.import_path)

        existing_files: list[Path] = []
        for file_path in import_path.iterdir():
            if file_path.is_file():
                ext = file_path.suffix.lower()
                if ext in ImageImportHandler.SUPPORTED_EXTENSIONS:
                    existing_files.append(file_path)

        if not existing_files:
            return

        logger.info(f"Processing {len(existing_files)} existing file(s)")

        # Lazily spin up the worker loop so the session factory is available.
        if self._loop_thread is None or not self._loop_thread.is_alive():
            self._loop_thread = threading.Thread(
                target=self._run_loop, daemon=True, name="WatcherLoop"
            )
            self._loop_thread.start()
            if not self._loop_ready.wait(timeout=5):
                raise RuntimeError("Watcher event loop failed to start")

        future = self._dispatch.submit(self._process_files_batch(existing_files))
        # Await completion from the calling loop (FastAPI lifespan)
        await asyncio.wrap_future(future)
