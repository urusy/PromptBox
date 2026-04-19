from typing import Annotated
from uuid import UUID

from fastapi import Cookie, Depends, HTTPException, Query, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.database import get_db
from app.schemas.image import ImageSearchParams, ImageSortBy, ImageSortOrder
from app.services.auth_service import AuthService

DbSession = Annotated[AsyncSession, Depends(get_db)]


async def get_current_user(
    session_token: str | None = Cookie(None, alias="session"),
) -> str:
    """Get the current authenticated user from session cookie."""
    if not session_token:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Not authenticated",
        )

    auth_service = AuthService()
    username = auth_service.verify_session(session_token)

    if not username:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid or expired session",
        )

    return username


CurrentUser = Annotated[str, Depends(get_current_user)]


def image_search_params(
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
    file_type: str | None = None,
    seed: int | None = None,
    seed_tolerance: int | None = Query(None, ge=0, le=1000),
    showcase_id: UUID | None = None,
    include_deleted: bool = False,
    page: int = Query(1, ge=1),
    per_page: int = Query(24, ge=1, le=120),
    sort_by: ImageSortBy = "created_at",
    sort_order: ImageSortOrder = "desc",
) -> ImageSearchParams:
    """Shared dependency: build ImageSearchParams from query string."""
    return ImageSearchParams(
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
        file_type=file_type,
        seed=seed,
        seed_tolerance=seed_tolerance,
        showcase_id=showcase_id,
        include_deleted=include_deleted,
        page=page,
        per_page=per_page,
        sort_by=sort_by,
        sort_order=sort_order,
    )


ImageSearchParamsDep = Annotated[ImageSearchParams, Depends(image_search_params)]
