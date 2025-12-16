from uuid import UUID

from fastapi import APIRouter, HTTPException, Query, status

from app.api.deps import CurrentUser, DbSession
from app.schemas.common import MessageResponse, PaginatedResponse
from app.schemas.image import (
    ImageListResponse,
    ImageResponse,
    ImageSearchParams,
    ImageUpdate,
)
from app.services.image_service import ImageService

router = APIRouter(prefix="/images", tags=["images"])


@router.get("", response_model=PaginatedResponse[ImageListResponse])
async def list_images(
    db: DbSession,
    _: CurrentUser,
    q: str | None = None,
    source_tool: str | None = None,
    model_type: str | None = None,
    model_name: str | None = None,
    sampler_name: str | None = None,
    min_rating: int | None = Query(None, ge=0, le=5),
    exact_rating: int | None = Query(None, ge=0, le=5),
    is_favorite: bool | None = None,
    needs_improvement: bool | None = None,
    tags: list[str] | None = Query(None),
    lora_name: str | None = None,
    is_xyz_grid: bool | None = None,
    is_upscaled: bool | None = None,
    orientation: str | None = None,
    min_width: int | None = Query(None, ge=1),
    min_height: int | None = Query(None, ge=1),
    date_from: str | None = None,
    seed: int | None = None,
    seed_tolerance: int | None = Query(None, ge=0, le=1000),
    include_deleted: bool = False,
    page: int = Query(1, ge=1),
    per_page: int = Query(24, ge=1, le=120),
    sort_by: str = "created_at",
    sort_order: str = "desc",
) -> PaginatedResponse[ImageListResponse]:
    """List images with search and pagination."""
    params = ImageSearchParams(
        q=q,
        source_tool=source_tool,
        model_type=model_type,
        model_name=model_name,
        sampler_name=sampler_name,
        min_rating=min_rating,
        exact_rating=exact_rating,
        is_favorite=is_favorite,
        needs_improvement=needs_improvement,
        tags=tags,
        lora_name=lora_name,
        is_xyz_grid=is_xyz_grid,
        is_upscaled=is_upscaled,
        orientation=orientation,
        min_width=min_width,
        min_height=min_height,
        date_from=date_from,
        seed=seed,
        seed_tolerance=seed_tolerance,
        include_deleted=include_deleted,
        page=page,
        per_page=per_page,
        sort_by=sort_by,
        sort_order=sort_order,
    )

    service = ImageService(db)
    return await service.list_images(params)


@router.get("/{image_id}", response_model=ImageResponse)
async def get_image(
    image_id: UUID,
    db: DbSession,
    _: CurrentUser,
    q: str | None = None,
    source_tool: str | None = None,
    model_type: str | None = None,
    model_name: str | None = None,
    sampler_name: str | None = None,
    min_rating: int | None = Query(None, ge=0, le=5),
    exact_rating: int | None = Query(None, ge=0, le=5),
    is_favorite: bool | None = None,
    needs_improvement: bool | None = None,
    tags: list[str] | None = Query(None),
    lora_name: str | None = None,
    is_xyz_grid: bool | None = None,
    is_upscaled: bool | None = None,
    orientation: str | None = None,
    min_width: int | None = Query(None, ge=1),
    min_height: int | None = Query(None, ge=1),
    seed: int | None = None,
    seed_tolerance: int | None = Query(None, ge=0, le=1000),
    include_deleted: bool = False,
    sort_by: str = "created_at",
    sort_order: str = "desc",
) -> ImageResponse:
    """Get a single image by ID with prev/next navigation based on search context."""
    service = ImageService(db)

    # Always build search params for prev/next navigation
    search_params = ImageSearchParams(
        q=q,
        source_tool=source_tool,
        model_type=model_type,
        model_name=model_name,
        sampler_name=sampler_name,
        min_rating=min_rating,
        exact_rating=exact_rating,
        is_favorite=is_favorite,
        needs_improvement=needs_improvement,
        tags=tags,
        lora_name=lora_name,
        is_xyz_grid=is_xyz_grid,
        is_upscaled=is_upscaled,
        orientation=orientation,
        min_width=min_width,
        min_height=min_height,
        seed=seed,
        seed_tolerance=seed_tolerance,
        include_deleted=include_deleted,
        sort_by=sort_by,
        sort_order=sort_order,
    )
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
