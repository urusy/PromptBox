import os
from pathlib import Path


def ensure_directory(path: str | Path) -> Path:
    """Ensure a directory exists, creating it if necessary.

    Args:
        path: Directory path

    Returns:
        Path object of the directory
    """
    directory = Path(path)
    directory.mkdir(parents=True, exist_ok=True)
    return directory


def safe_path_join(base: str | Path, *paths: str) -> Path:
    """Safely join paths, preventing directory traversal attacks.

    Args:
        base: Base directory path
        *paths: Additional path components

    Returns:
        Joined path

    Raises:
        ValueError: If the resulting path is outside the base directory
    """
    base_path = Path(base).resolve()
    joined = base_path.joinpath(*paths).resolve()

    # Ensure the joined path is within the base directory
    if not str(joined).startswith(str(base_path)):
        raise ValueError("Path traversal detected")

    return joined


def get_storage_path(storage_base: str, uuid: str, filename: str) -> str:
    """Generate a storage path for an image using UUID v7 partitioning.

    Args:
        storage_base: Base storage directory
        uuid: UUID v7 string
        filename: Original filename

    Returns:
        Relative storage path

    Example:
        UUID: 01935a2b-3c4d-7e8f-9a0b-1c2d3e4f5a6b
        Result: 01/93/5a2b-3c4d-7e8f-9a0b-1c2d3e4f5a6b.png
    """
    # Use first 4 characters for directory partitioning
    prefix1 = uuid[:2]
    prefix2 = uuid[2:4]
    ext = Path(filename).suffix.lower()

    return os.path.join(prefix1, prefix2, f"{uuid}{ext}")


def get_thumbnail_path(storage_base: str, uuid: str) -> str:
    """Generate a thumbnail path for an image.

    Args:
        storage_base: Base storage directory
        uuid: UUID v7 string

    Returns:
        Relative thumbnail path
    """
    prefix1 = uuid[:2]
    prefix2 = uuid[2:4]

    return os.path.join("thumbnails", prefix1, prefix2, f"{uuid}.webp")
