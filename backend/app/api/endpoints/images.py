from uuid import UUID

from fastapi import APIRouter, HTTPException, status

from app.api.deps import CurrentUser, DbSession, ImageSearchParamsDep
from app.schemas.common import MessageResponse, PaginatedResponse
from app.schemas.image import (
    ImageListResponse,
    ImageResponse,
    ImageUpdate,
)
from app.services.image_service import ImageService

router = APIRouter(prefix="/images", tags=["images"])


@router.get("", response_model=PaginatedResponse[ImageListResponse])
async def list_images(
    db: DbSession,
    _: CurrentUser,
    params: ImageSearchParamsDep,
) -> PaginatedResponse[ImageListResponse]:
    """List images with search and pagination."""
    service = ImageService(db)
    return await service.list_images(params)


@router.get("/{image_id}", response_model=ImageResponse)
async def get_image(
    image_id: UUID,
    db: DbSession,
    _: CurrentUser,
    search_params: ImageSearchParamsDep,
) -> ImageResponse:
    """Get a single image by ID with prev/next navigation based on search context."""
    service = ImageService(db)
    image = await service.get_image_with_neighbors(image_id, search_params)

    if not image:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Image not found",
        )

    return image


@router.patch("/{image_id}", response_model=ImageResponse)
async def update_image(
    image_id: UUID,
    update_data: ImageUpdate,
    db: DbSession,
    _: CurrentUser,
) -> ImageResponse:
    """Update image metadata (rating, tags, etc.)."""
    service = ImageService(db)
    image = await service.update_image(image_id, update_data)

    if not image:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Image not found",
        )

    return image


@router.delete("/{image_id}", response_model=MessageResponse)
async def delete_image(
    image_id: UUID,
    db: DbSession,
    _: CurrentUser,
    permanent: bool = False,
) -> MessageResponse:
    """Delete an image (soft delete by default, permanent if specified)."""
    service = ImageService(db)
    success = await service.delete_image(image_id, permanent=permanent)

    if not success:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Image not found",
        )

    action = "permanently deleted" if permanent else "moved to trash"
    return MessageResponse(message=f"Image {action}")


@router.post("/{image_id}/restore", response_model=MessageResponse)
async def restore_image(
    image_id: UUID,
    db: DbSession,
    _: CurrentUser,
) -> MessageResponse:
    """Restore a soft-deleted image."""
    service = ImageService(db)
    success = await service.restore_image(image_id)

    if not success:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Image not found or not deleted",
        )

    return MessageResponse(message="Image restored")
