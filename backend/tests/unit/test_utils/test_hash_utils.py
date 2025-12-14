"""Tests for hash utility functions."""

from pathlib import Path

import pytest

from app.utils.hash_utils import calculate_file_hash


class TestHashUtils:
    """Test cases for hash utilities."""

    def test_calculate_file_hash_sha256(self, temp_dir: Path) -> None:
        """Test SHA-256 hash calculation."""
        test_file = temp_dir / "test.txt"
        test_file.write_text("Hello, World!")

        result = calculate_file_hash(test_file)

        # Known SHA-256 hash for "Hello, World!"
        expected = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        assert result == expected

    def test_calculate_file_hash_md5(self, temp_dir: Path) -> None:
        """Test MD5 hash calculation."""
        test_file = temp_dir / "test.txt"
        test_file.write_text("Hello, World!")

        result = calculate_file_hash(test_file, algorithm="md5")

        # Known MD5 hash for "Hello, World!"
        expected = "65a8e27d8879283831b664bd8b7f0ad4"
        assert result == expected

    def test_calculate_file_hash_empty_file(self, temp_dir: Path) -> None:
        """Test hash calculation for empty file."""
        test_file = temp_dir / "empty.txt"
        test_file.write_text("")

        result = calculate_file_hash(test_file)

        # Known SHA-256 hash for empty string
        expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        assert result == expected

    def test_calculate_file_hash_large_file(self, temp_dir: Path) -> None:
        """Test hash calculation for larger file (uses chunking)."""
        test_file = temp_dir / "large.txt"
        # Create a file larger than the chunk size (8192 bytes)
        content = "x" * 20000
        test_file.write_text(content)

        result = calculate_file_hash(test_file)

        # Should produce a valid 64-character hex string
        assert len(result) == 64
        assert all(c in "0123456789abcdef" for c in result)

    def test_calculate_file_hash_binary_file(self, temp_dir: Path) -> None:
        """Test hash calculation for binary file."""
        test_file = temp_dir / "binary.bin"
        test_file.write_bytes(bytes([0x00, 0xFF, 0x10, 0x20]))

        result = calculate_file_hash(test_file)

        # Should produce a valid 64-character hex string
        assert len(result) == 64
        assert all(c in "0123456789abcdef" for c in result)

    def test_calculate_file_hash_string_path(self, temp_dir: Path) -> None:
        """Test hash calculation with string path."""
        test_file = temp_dir / "test.txt"
        test_file.write_text("test content")

        result = calculate_file_hash(str(test_file))

        assert len(result) == 64

    def test_calculate_file_hash_nonexistent_file(self, temp_dir: Path) -> None:
        """Test hash calculation for nonexistent file raises error."""
        nonexistent = temp_dir / "does_not_exist.txt"

        with pytest.raises(FileNotFoundError):
            calculate_file_hash(nonexistent)
