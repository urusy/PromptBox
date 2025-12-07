import hashlib
from pathlib import Path


def calculate_file_hash(file_path: str | Path, algorithm: str = "sha256") -> str:
    """Calculate the hash of a file.

    Args:
        file_path: Path to the file
        algorithm: Hash algorithm to use (default: sha256)

    Returns:
        Hex digest of the file hash
    """
    hash_func = hashlib.new(algorithm)
    path = Path(file_path)

    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            hash_func.update(chunk)

    return hash_func.hexdigest()
