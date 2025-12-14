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

    This function resolves symlinks and ensures the final path is within
    the base directory, protecting against both path traversal (../) and
    symlink attacks.

    Args:
        base: Base directory path
        *paths: Additional path components

    Returns:
        Joined path

    Raises:
        ValueError: If the resulting path is outside the base directory
    """
    base_path = Path(base).resolve()
    joined = base_path.joinpath(*paths)

    # Resolve symlinks after joining to detect symlink attacks
    resolved = joined.resolve()

    # Use is_relative_to for robust containment check (Python 3.9+)
    try:
        resolved.relative_to(base_path)
    except ValueError as e:
        raise ValueError("Path traversal detected") from e

    return resolved


def get_storage_path(storage_base: str, file_hash: str, filename: str) -> str:
    """Generate a storage path for an image using content hash partitioning.

    Args:
        storage_base: Base storage directory
        file_hash: SHA256 hash of the file content
        filename: Original filename (for extension)

    Returns:
        Relative storage path

    Example:
        Hash: a1b2c3d4e5f6...
        Result: a1/b2/a1b2c3d4e5f6....png
    """
    # Use first 4 characters for directory partitioning
    prefix1 = file_hash[:2]
    prefix2 = file_hash[2:4]
    ext = Path(filename).suffix.lower()

    return os.path.join(prefix1, prefix2, f"{file_hash}{ext}")


def get_thumbnail_path(storage_base: str, file_hash: str) -> str:
    """Generate a thumbnail path for an image.

    Args:
        storage_base: Base storage directory
        file_hash: SHA256 hash of the file content

    Returns:
        Relative thumbnail path
    """
    prefix1 = file_hash[:2]
    prefix2 = file_hash[2:4]

    return os.path.join("thumbnails", prefix1, prefix2, f"{file_hash}.webp")
