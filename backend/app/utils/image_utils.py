from pathlib import Path

from PIL import Image


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
    """Create a WebP thumbnail of an image.

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
