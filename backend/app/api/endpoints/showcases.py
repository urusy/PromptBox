from datetime import datetime
from uuid import UUID

from fastapi import APIRouter, HTTPException, status
from sqlalchemy import delete, func, select
from uuid_utils import uuid7

from app.api.deps import CurrentUser, DbSession
from app.models.image import Image
from app.models.showcase import Showcase, ShowcaseImage
from app.schemas.common import MessageResponse
from app.schemas.showcase import (
    ShowcaseCreate,
    ShowcaseDetailResponse,
    ShowcaseImageAdd,
    ShowcaseImageCheck,
    ShowcaseImageCheckResult,
    ShowcaseImageInfo,
    ShowcaseImageRemove,
    ShowcaseImageReorder,
    ShowcaseResponse,
    ShowcaseUpdate,
)

router = APIRouter(prefix="/showcases", tags=["showcases"])


@router.get("", response_model=list[ShowcaseResponse])
async def list_showcases(
    db: DbSession,
    _: CurrentUser,
) -> list[ShowcaseResponse]:
    """List all showcases with image counts."""
    # Get showcases with image counts
    count_subquery = (
        select(
            ShowcaseImage.showcase_id,
            func.count(ShowcaseImage.image_id).label("image_count"),
        )
        .group_by(ShowcaseImage.showcase_id)
        .subquery()
    )

    result = await db.execute(
        select(
            Showcase,
            func.coalesce(count_subquery.c.image_count, 0).label("image_count"),
            Image.thumbnail_path,
        )
        .outerjoin(count_subquery, Showcase.id == count_subquery.c.showcase_id)
        .outerjoin(Image, Showcase.cover_image_id == Image.id)
        .order_by(Showcase.created_at.desc())
    )

    showcases = []
    for row in result:
        showcase = row[0]
        image_count = row[1]
        cover_thumbnail_path = row[2]
        showcases.append(
            ShowcaseResponse(
                id=showcase.id,
                name=showcase.name,
                description=showcase.description,
                icon=showcase.icon,
                cover_image_id=showcase.cover_image_id,
                cover_thumbnail_path=cover_thumbnail_path,
                image_count=image_count,
                created_at=showcase.created_at,
                updated_at=showcase.updated_at,
            )
        )

    return showcases


@router.post("/check-images", response_model=list[ShowcaseImageCheckResult])
async def check_images_in_showcases(
    db: DbSession,
    _: CurrentUser,
    data: ShowcaseImageCheck,
) -> list[ShowcaseImageCheckResult]:
    """Check which images are already in each showcase."""
    # Get count of matching images per showcase
    result = await db.execute(
        select(
            ShowcaseImage.showcase_id,
            func.count(ShowcaseImage.image_id).label("existing_count"),
        )
        .where(ShowcaseImage.image_id.in_(data.image_ids))
        .group_by(ShowcaseImage.showcase_id)
    )

    return [
        ShowcaseImageCheckResult(
            showcase_id=row[0],
            existing_count=row[1],
        )
        for row in result
    ]


@router.post("", response_model=ShowcaseResponse, status_code=status.HTTP_201_CREATED)
async def create_showcase(
    db: DbSession,
    _: CurrentUser,
    data: ShowcaseCreate,
) -> ShowcaseResponse:
    """Create a new showcase."""
    showcase = Showcase(
        id=uuid7(),
        name=data.name,
        description=data.description,
        icon=data.icon,
        created_at=datetime.utcnow(),
        updated_at=datetime.utcnow(),
    )
    db.add(showcase)
    await db.commit()
    await db.refresh(showcase)

    return ShowcaseResponse(
        id=showcase.id,
        name=showcase.name,
        description=showcase.description,
        icon=showcase.icon,
        cover_image_id=showcase.cover_image_id,
        cover_thumbnail_path=None,
        image_count=0,
        created_at=showcase.created_at,
        updated_at=showcase.updated_at,
    )


@router.get("/{showcase_id}", response_model=ShowcaseDetailResponse)
async def get_showcase(
    db: DbSession,
    _: CurrentUser,
    showcase_id: UUID,
) -> ShowcaseDetailResponse:
    """Get a showcase by ID with its images."""
    result = await db.execute(
        select(Showcase, Image.thumbnail_path)
        .outerjoin(Image, Showcase.cover_image_id == Image.id)
        .where(Showcase.id == showcase_id)
    )
    row = result.one_or_none()

    if not row:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Showcase not found",
        )

    showcase = row[0]
    cover_thumbnail_path = row[1]

    # Get images in this showcase
    images_result = await db.execute(
        select(
            Image.id,
            Image.storage_path,
            Image.thumbnail_path,
            ShowcaseImage.sort_order,
            ShowcaseImage.added_at,
        )
        .join(ShowcaseImage, Image.id == ShowcaseImage.image_id)
        .where(ShowcaseImage.showcase_id == showcase_id)
        .where(Image.deleted_at.is_(None))
        .order_by(ShowcaseImage.sort_order)
    )

    images = [
        ShowcaseImageInfo(
            id=row[0],
            storage_path=row[1],
            thumbnail_path=row[2],
            sort_order=row[3],
            added_at=row[4],
        )
        for row in images_result
    ]

    return ShowcaseDetailResponse(
        id=showcase.id,
        name=showcase.name,
        description=showcase.description,
        icon=showcase.icon,
        cover_image_id=showcase.cover_image_id,
        cover_thumbnail_path=cover_thumbnail_path,
        image_count=len(images),
        created_at=showcase.created_at,
        updated_at=showcase.updated_at,
        images=images,
    )


@router.put("/{showcase_id}", response_model=ShowcaseResponse)
async def update_showcase(
    db: DbSession,
    _: CurrentUser,
    showcase_id: UUID,
    data: ShowcaseUpdate,
) -> ShowcaseResponse:
    """Update an existing showcase."""
    result = await db.execute(select(Showcase).where(Showcase.id == showcase_id))
    showcase = result.scalar_one_or_none()

    if not showcase:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Showcase not found",
        )

    if data.name is not None:
        showcase.name = data.name
    if data.description is not None:
        showcase.description = data.description if data.description else None
    if data.icon is not None:
        showcase.icon = data.icon if data.icon else None
    if data.cover_image_id is not None:
        showcase.cover_image_id = data.cover_image_id

    showcase.updated_at = datetime.utcnow()

    await db.commit()
    await db.refresh(showcase)

    # Get image count and cover thumbnail path
    count_result = await db.execute(
        select(func.count(ShowcaseImage.image_id)).where(
            ShowcaseImage.showcase_id == showcase_id
        )
    )
    image_count = count_result.scalar() or 0

    cover_thumbnail_path = None
    if showcase.cover_image_id:
        cover_result = await db.execute(
            select(Image.thumbnail_path).where(Image.id == showcase.cover_image_id)
        )
        cover_thumbnail_path = cover_result.scalar_one_or_none()

    return ShowcaseResponse(
        id=showcase.id,
        name=showcase.name,
        description=showcase.description,
        icon=showcase.icon,
        cover_image_id=showcase.cover_image_id,
        cover_thumbnail_path=cover_thumbnail_path,
        image_count=image_count,
        created_at=showcase.created_at,
        updated_at=showcase.updated_at,
    )


@router.delete("/{showcase_id}", response_model=MessageResponse)
async def delete_showcase(
    db: DbSession,
    _: CurrentUser,
    showcase_id: UUID,
) -> MessageResponse:
    """Delete a showcase."""
    result = await db.execute(select(Showcase).where(Showcase.id == showcase_id))
    showcase = result.scalar_one_or_none()

    if not showcase:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Showcase not found",
        )

    await db.delete(showcase)
    await db.commit()
    return MessageResponse(message="Showcase deleted successfully")


@router.post("/{showcase_id}/images", response_model=MessageResponse)
async def add_images_to_showcase(
    db: DbSession,
    _: CurrentUser,
    showcase_id: UUID,
    data: ShowcaseImageAdd,
) -> MessageResponse:
    """Add images to a showcase."""
    # Verify showcase exists
    result = await db.execute(select(Showcase).where(Showcase.id == showcase_id))
    showcase = result.scalar_one_or_none()

    if not showcase:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Showcase not found",
        )

    # Get max sort_order
    max_order_result = await db.execute(
        select(func.max(ShowcaseImage.sort_order)).where(
            ShowcaseImage.showcase_id == showcase_id
        )
    )
    max_order = max_order_result.scalar() or 0

    # Get existing image IDs in this showcase
    existing_result = await db.execute(
        select(ShowcaseImage.image_id).where(ShowcaseImage.showcase_id == showcase_id)
    )
    existing_ids = {row[0] for row in existing_result}

    # Add new images
    added_count = 0
    for image_id in data.image_ids:
        if image_id not in existing_ids:
            max_order += 1
            showcase_image = ShowcaseImage(
                showcase_id=showcase_id,
                image_id=image_id,
                sort_order=max_order,
                added_at=datetime.utcnow(),
            )
            db.add(showcase_image)
            added_count += 1

    # Update showcase's updated_at
    showcase.updated_at = datetime.utcnow()

    # Set cover image if not set
    if showcase.cover_image_id is None and data.image_ids:
        showcase.cover_image_id = data.image_ids[0]

    await db.commit()
    return MessageResponse(message=f"Added {added_count} images to showcase")


@router.delete("/{showcase_id}/images", response_model=MessageResponse)
async def remove_images_from_showcase(
    db: DbSession,
    _: CurrentUser,
    showcase_id: UUID,
    data: ShowcaseImageRemove,
) -> MessageResponse:
    """Remove images from a showcase."""
    # Verify showcase exists
    result = await db.execute(select(Showcase).where(Showcase.id == showcase_id))
    showcase = result.scalar_one_or_none()

    if not showcase:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Showcase not found",
        )

    # Delete the associations
    delete_result = await db.execute(
        delete(ShowcaseImage).where(
            ShowcaseImage.showcase_id == showcase_id,
            ShowcaseImage.image_id.in_(data.image_ids),
        )
    )

    # If cover image was removed, clear it
    if showcase.cover_image_id in data.image_ids:
        showcase.cover_image_id = None

    # Update showcase's updated_at
    showcase.updated_at = datetime.utcnow()

    await db.commit()
    return MessageResponse(
        message=f"Removed {delete_result.rowcount} images from showcase"
    )


@router.put("/{showcase_id}/images/reorder", response_model=MessageResponse)
async def reorder_images_in_showcase(
    db: DbSession,
    _: CurrentUser,
    showcase_id: UUID,
    data: ShowcaseImageReorder,
) -> MessageResponse:
    """Reorder images in a showcase."""
    # Verify showcase exists
    result = await db.execute(select(Showcase).where(Showcase.id == showcase_id))
    showcase = result.scalar_one_or_none()

    if not showcase:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Showcase not found",
        )

    # Update sort_order for each image
    for i, image_id in enumerate(data.image_ids):
        await db.execute(
            ShowcaseImage.__table__.update()
            .where(ShowcaseImage.showcase_id == showcase_id)
            .where(ShowcaseImage.image_id == image_id)
            .values(sort_order=i)
        )

    # Update showcase's updated_at
    showcase.updated_at = datetime.utcnow()

    await db.commit()
    return MessageResponse(message="Images reordered successfully")
