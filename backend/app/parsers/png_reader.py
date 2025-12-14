import logging
from pathlib import Path
from typing import Any

from PIL import Image

logger = logging.getLogger(__name__)


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


def read_jpeg_info(file_path: str | Path) -> dict[str, Any]:
    """Read metadata from a JPEG file.

    A1111/Forge stores generation parameters in the EXIF UserComment field.

    Args:
        file_path: Path to the JPEG file

    Returns:
        Dictionary containing JPEG metadata (converted to A1111 format)
    """
    with Image.open(file_path) as img:
        jpeg_info: dict[str, Any] = {}

        # Check if there's EXIF data
        if "exif" not in img.info:
            return jpeg_info

        try:
            exif = img.getexif()
            if not exif:
                return jpeg_info

            # EXIF tag 37510 is UserComment where A1111 stores parameters
            # We can also check tag 0x9286 (UserComment in IFD)

            # The UserComment (0x9286) is in the Exif IFD
            exif_ifd = exif.get_ifd(0x8769)
            if exif_ifd and 0x9286 in exif_ifd:
                user_comment = exif_ifd[0x9286]
                if isinstance(user_comment, bytes):
                    # UserComment starts with charset marker (8 bytes)
                    # Common markers: "UNICODE\0" or "ASCII\0\0\0"
                    text = ""
                    if user_comment.startswith(b"UNICODE\x00"):
                        # UTF-16 Big Endian (A1111 uses this format)
                        text = user_comment[8:].decode("utf-16-be", errors="ignore")
                    elif user_comment.startswith(b"ASCII\x00\x00\x00"):
                        text = user_comment[8:].decode("ascii", errors="ignore")
                    else:
                        # Try UTF-8 as fallback
                        text = user_comment.decode("utf-8", errors="ignore")

                    # A1111 format: parameters are all in one string
                    if text and ("Steps:" in text or "Sampler:" in text):
                        jpeg_info["parameters"] = text.strip()

        except Exception as e:
            # If EXIF parsing fails, return empty dict
            logger.debug(f"Failed to parse EXIF data from {file_path}: {e}")

        return jpeg_info


def read_image_info(file_path: str | Path) -> dict[str, Any]:
    """Read metadata from an image file (PNG or JPEG).

    Args:
        file_path: Path to the image file

    Returns:
        Dictionary containing image metadata
    """
    file_path = Path(file_path)
    suffix = file_path.suffix.lower()

    if suffix == ".png":
        return read_png_info(file_path)
    elif suffix in (".jpg", ".jpeg"):
        return read_jpeg_info(file_path)
    else:
        return {}
