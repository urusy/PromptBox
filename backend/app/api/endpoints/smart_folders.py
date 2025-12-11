from datetime import datetime
from uuid import UUID

from fastapi import APIRouter, HTTPException, status
from sqlalchemy import select
from uuid_utils import uuid7

from app.api.deps import CurrentUser, DbSession
from app.models.smart_folder import SmartFolder
from app.schemas.common import MessageResponse
from app.schemas.smart_folder import (
    SmartFolderCreate,
    SmartFolderResponse,
    SmartFolderUpdate,
)

router = APIRouter(prefix="/smart-folders", tags=["smart-folders"])


@router.get("", response_model=list[SmartFolderResponse])
async def list_smart_folders(
    db: DbSession,
    _: CurrentUser,
) -> list[SmartFolderResponse]:
    """List all smart folders ordered by created_at descending."""
    result = await db.execute(
        select(SmartFolder).order_by(SmartFolder.created_at.desc())
    )
    folders = result.scalars().all()
    return [SmartFolderResponse.model_validate(f) for f in folders]


@router.post("", response_model=SmartFolderResponse, status_code=status.HTTP_201_CREATED)
async def create_smart_folder(
    db: DbSession,
    _: CurrentUser,
    data: SmartFolderCreate,
) -> SmartFolderResponse:
    """Create a new smart folder."""
    folder = SmartFolder(
        id=uuid7(),
        name=data.name,
        icon=data.icon,
        filters=data.filters.model_dump(exclude_none=True),
        created_at=datetime.utcnow(),
        updated_at=datetime.utcnow(),
    )
    db.add(folder)
    await db.commit()
    await db.refresh(folder)
    return SmartFolderResponse.model_validate(folder)


@router.get("/{folder_id}", response_model=SmartFolderResponse)
async def get_smart_folder(
    db: DbSession,
    _: CurrentUser,
    folder_id: UUID,
) -> SmartFolderResponse:
    """Get a smart folder by ID."""
    result = await db.execute(select(SmartFolder).where(SmartFolder.id == folder_id))
    folder = result.scalar_one_or_none()

    if not folder:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Smart folder not found",
        )

    return SmartFolderResponse.model_validate(folder)


@router.put("/{folder_id}", response_model=SmartFolderResponse)
async def update_smart_folder(
    db: DbSession,
    _: CurrentUser,
    folder_id: UUID,
    data: SmartFolderUpdate,
) -> SmartFolderResponse:
    """Update an existing smart folder."""
    result = await db.execute(select(SmartFolder).where(SmartFolder.id == folder_id))
    folder = result.scalar_one_or_none()

    if not folder:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Smart folder not found",
        )

    if data.name is not None:
        folder.name = data.name
    if data.icon is not None:
        folder.icon = data.icon
    if data.filters is not None:
        folder.filters = data.filters.model_dump(exclude_none=True)
    folder.updated_at = datetime.utcnow()

    await db.commit()
    await db.refresh(folder)
    return SmartFolderResponse.model_validate(folder)


@router.delete("/{folder_id}", response_model=MessageResponse)
async def delete_smart_folder(
    db: DbSession,
    _: CurrentUser,
    folder_id: UUID,
) -> MessageResponse:
    """Delete a smart folder."""
    result = await db.execute(select(SmartFolder).where(SmartFolder.id == folder_id))
    folder = result.scalar_one_or_none()

    if not folder:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Smart folder not found",
        )

    await db.delete(folder)
    await db.commit()
    return MessageResponse(message="Smart folder deleted successfully")
