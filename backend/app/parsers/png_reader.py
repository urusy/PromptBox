from pathlib import Path
from typing import Any

from PIL import Image


def read_png_info(file_path: str | Path) -> dict[str, Any]:
    """Read metadata from a PNG file.

    Args:
        file_path: Path to the PNG file

    Returns:
        Dictionary containing PNG metadata
    """
    with Image.open(file_path) as img:
        png_info: dict[str, Any] = {}

        if hasattr(img, "info"):
            for key, value in img.info.items():
                png_info[key] = value

        return png_info
