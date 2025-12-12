import asyncio
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from PIL import Image

# Thread pool for image processing (Pillow is not async)
_image_executor = ThreadPoolExecutor(max_workers=2, thread_name_prefix="image_worker")


def get_image_dimensions(file_path: str | Path) -> tuple[int, int]:
    """Get the dimensions of an image.

    Args:
        file_path: Path to the image file

    Returns:
        Tuple of (width, height)
    """
    with Image.open(file_path) as img:
        return img.size


def create_thumbnail(
    source_path: str | Path,
    dest_path: str | Path,
    size: int = 300,
    quality: int = 85,
) -> None:
    """Create a WebP thumbnail of an image (synchronous version).

    Args:
        source_path: Path to the source image
        dest_path: Path to save the thumbnail
        size: Maximum size of the thumbnail (default: 300)
        quality: WebP quality (default: 85)
    """
    dest = Path(dest_path)
    dest.parent.mkdir(parents=True, exist_ok=True)

    with Image.open(source_path) as img:
        # Convert to RGB if necessary (for PNG with transparency)
        if img.mode in ("RGBA", "P"):
            img = img.convert("RGB")

        # Calculate thumbnail size maintaining aspect ratio
        img.thumbnail((size, size), Image.Resampling.LANCZOS)

        # Save as WebP
        img.save(dest, "WebP", quality=quality)


async def create_thumbnail_async(
    source_path: str | Path,
    dest_path: str | Path,
    size: int = 300,
    quality: int = 85,
) -> None:
    """Create a WebP thumbnail of an image (async version).

    Uses a thread pool to avoid blocking the event loop.

    Args:
        source_path: Path to the source image
        dest_path: Path to save the thumbnail
        size: Maximum size of the thumbnail (default: 300)
        quality: WebP quality (default: 85)
    """
    loop = asyncio.get_event_loop()
    await loop.run_in_executor(
        _image_executor,
        create_thumbnail,
        source_path,
        dest_path,
        size,
        quality,
    )


async def get_image_dimensions_async(file_path: str | Path) -> tuple[int, int]:
    """Get the dimensions of an image (async version).

    Uses a thread pool to avoid blocking the event loop.

    Args:
        file_path: Path to the image file

    Returns:
        Tuple of (width, height)
    """
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(
        _image_executor,
        get_image_dimensions,
        file_path,
    )
