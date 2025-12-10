from uuid import UUID

from fastapi import APIRouter, HTTPException, status
from pydantic import BaseModel, Field

from app.api.deps import CurrentUser, DbSession
from app.schemas.common import MessageResponse
from app.services.batch_service import BatchService

router = APIRouter(prefix="/bulk", tags=["bulk"])


class BatchUpdateRequest(BaseModel):
    ids: list[UUID] = Field(..., min_length=1, max_length=100)
    rating: int | None = Field(None, ge=0, le=5)
    is_favorite: bool | None = None
    needs_improvement: bool | None = None
    add_tags: list[str] | None = None
    remove_tags: list[str] | None = None


class BatchDeleteRequest(BaseModel):
    ids: list[UUID] = Field(..., min_length=1, max_length=100)
    permanent: bool = False


class BatchRestoreRequest(BaseModel):
    ids: list[UUID] = Field(..., min_length=1, max_length=100)


@router.post("/update", response_model=MessageResponse)
async def batch_update(
    request: BatchUpdateRequest,
    db: DbSession,
    _: CurrentUser,
) -> MessageResponse:
    """Update multiple images at once."""
    service = BatchService(db)
    count = await service.batch_update(
        ids=request.ids,
        rating=request.rating,
        is_favorite=request.is_favorite,
        needs_improvement=request.needs_improvement,
        add_tags=request.add_tags,
        remove_tags=request.remove_tags,
    )

    if count == 0:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="No images found",
        )

    return MessageResponse(message=f"Updated {count} images")


@router.post("/delete", response_model=MessageResponse)
async def batch_delete(
    request: BatchDeleteRequest,
    db: DbSession,
    _: CurrentUser,
) -> MessageResponse:
    """Delete multiple images at once."""
    service = BatchService(db)
    count = await service.batch_delete(request.ids, permanent=request.permanent)

    if count == 0:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="No images found",
        )

    action = "permanently deleted" if request.permanent else "moved to trash"
    return MessageResponse(message=f"{count} images {action}")


@router.post("/restore", response_model=MessageResponse)
async def batch_restore(
    request: BatchRestoreRequest,
    db: DbSession,
    _: CurrentUser,
) -> MessageResponse:
    """Restore multiple deleted images."""
    service = BatchService(db)
    count = await service.batch_restore(request.ids)

    if count == 0:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="No deleted images found",
        )

    return MessageResponse(message=f"Restored {count} images")
