import shutil
from pathlib import Path

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

from app.api.deps import CurrentUser
from app.config import get_settings

router = APIRouter(prefix="/duplicates", tags=["duplicates"])


class DuplicatesInfo(BaseModel):
    count: int
    total_size_bytes: int
    files: list[str]


class DeleteResult(BaseModel):
    deleted_count: int
    freed_bytes: int


def get_duplicated_dir() -> Path:
    """Get the duplicated files directory path."""
    settings = get_settings()
    return Path(settings.import_path) / "duplicated"


@router.get("", response_model=DuplicatesInfo)
async def get_duplicates_info(current_user: CurrentUser) -> DuplicatesInfo:
    """Get information about duplicated files."""
    duplicated_dir = get_duplicated_dir()

    if not duplicated_dir.exists():
        return DuplicatesInfo(count=0, total_size_bytes=0, files=[])

    files: list[str] = []
    total_size = 0

    for file_path in duplicated_dir.iterdir():
        if file_path.is_file() and not file_path.name.startswith("."):
            files.append(file_path.name)
            total_size += file_path.stat().st_size

    # Sort by name
    files.sort()

    return DuplicatesInfo(
        count=len(files),
        total_size_bytes=total_size,
        files=files,
    )


@router.delete("", response_model=DeleteResult)
async def delete_all_duplicates(current_user: CurrentUser) -> DeleteResult:
    """Delete all duplicated files."""
    duplicated_dir = get_duplicated_dir()

    if not duplicated_dir.exists():
        return DeleteResult(deleted_count=0, freed_bytes=0)

    deleted_count = 0
    freed_bytes = 0

    for file_path in duplicated_dir.iterdir():
        if file_path.is_file() and not file_path.name.startswith("."):
            try:
                freed_bytes += file_path.stat().st_size
                file_path.unlink()
                deleted_count += 1
            except Exception:
                pass

    return DeleteResult(deleted_count=deleted_count, freed_bytes=freed_bytes)


@router.delete("/{filename}")
async def delete_duplicate_file(filename: str, current_user: CurrentUser) -> dict:
    """Delete a specific duplicated file."""
    duplicated_dir = get_duplicated_dir()
    file_path = duplicated_dir / filename

    # Security check - prevent path traversal
    if not file_path.resolve().parent == duplicated_dir.resolve():
        raise HTTPException(status_code=400, detail="Invalid filename")

    if not file_path.exists():
        raise HTTPException(status_code=404, detail="File not found")

    try:
        file_size = file_path.stat().st_size
        file_path.unlink()
        return {"deleted": filename, "freed_bytes": file_size}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
