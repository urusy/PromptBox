from datetime import datetime
from uuid import UUID

from fastapi import APIRouter, HTTPException, status
from sqlalchemy import select
from uuid_utils import uuid7

from app.api.deps import CurrentUser, DbSession
from app.models.search_preset import SearchPreset
from app.schemas.common import MessageResponse
from app.schemas.search_preset import (
    SearchPresetCreate,
    SearchPresetResponse,
    SearchPresetUpdate,
)

router = APIRouter(prefix="/search-presets", tags=["search-presets"])


@router.get("", response_model=list[SearchPresetResponse])
async def list_search_presets(
    db: DbSession,
    _: CurrentUser,
) -> list[SearchPresetResponse]:
    """List all search presets ordered by created_at descending."""
    result = await db.execute(
        select(SearchPreset).order_by(SearchPreset.created_at.desc())
    )
    presets = result.scalars().all()
    return [SearchPresetResponse.model_validate(p) for p in presets]


@router.post("", response_model=SearchPresetResponse, status_code=status.HTTP_201_CREATED)
async def create_search_preset(
    db: DbSession,
    _: CurrentUser,
    data: SearchPresetCreate,
) -> SearchPresetResponse:
    """Create a new search preset."""
    preset = SearchPreset(
        id=uuid7(),
        name=data.name,
        filters=data.filters.model_dump(exclude_none=True),
        created_at=datetime.utcnow(),
        updated_at=datetime.utcnow(),
    )
    db.add(preset)
    await db.commit()
    await db.refresh(preset)
    return SearchPresetResponse.model_validate(preset)


@router.put("/{preset_id}", response_model=SearchPresetResponse)
async def update_search_preset(
    db: DbSession,
    _: CurrentUser,
    preset_id: UUID,
    data: SearchPresetUpdate,
) -> SearchPresetResponse:
    """Update an existing search preset."""
    result = await db.execute(select(SearchPreset).where(SearchPreset.id == preset_id))
    preset = result.scalar_one_or_none()

    if not preset:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Search preset not found",
        )

    if data.name is not None:
        preset.name = data.name
    if data.filters is not None:
        preset.filters = data.filters.model_dump(exclude_none=True)
    preset.updated_at = datetime.utcnow()

    await db.commit()
    await db.refresh(preset)
    return SearchPresetResponse.model_validate(preset)


@router.delete("/{preset_id}", response_model=MessageResponse)
async def delete_search_preset(
    db: DbSession,
    _: CurrentUser,
    preset_id: UUID,
) -> MessageResponse:
    """Delete a search preset."""
    result = await db.execute(select(SearchPreset).where(SearchPreset.id == preset_id))
    preset = result.scalar_one_or_none()

    if not preset:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Search preset not found",
        )

    await db.delete(preset)
    await db.commit()
    return MessageResponse(message="Search preset deleted successfully")
