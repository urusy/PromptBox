from datetime import datetime
from uuid import UUID

from sqlalchemy import func, select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.image import Image
from app.schemas.common import PaginatedResponse
from app.schemas.image import ImageListResponse, ImageResponse, ImageSearchParams, ImageUpdate


class ImageService:
    def __init__(self, db: AsyncSession) -> None:
        self.db = db

    async def list_images(
        self, params: ImageSearchParams
    ) -> PaginatedResponse[ImageListResponse]:
        """List images with search and pagination."""
        query = select(Image)

        # Filter by deleted status
        if not params.include_deleted:
            query = query.where(Image.deleted_at.is_(None))
        else:
            query = query.where(Image.deleted_at.is_not(None))

        # Apply filters
        if params.source_tool:
            query = query.where(Image.source_tool == params.source_tool)

        if params.model_type:
            query = query.where(Image.model_type == params.model_type)

        if params.model_name:
            query = query.where(Image.model_name.ilike(f"%{params.model_name}%"))

        if params.sampler_name:
            query = query.where(Image.sampler_name == params.sampler_name)

        if params.min_rating is not None:
            query = query.where(Image.rating >= params.min_rating)

        if params.is_favorite is not None:
            query = query.where(Image.is_favorite == params.is_favorite)

        if params.needs_improvement is not None:
            query = query.where(Image.needs_improvement == params.needs_improvement)

        if params.tags:
            for tag in params.tags:
                query = query.where(Image.user_tags.contains([tag]))

        if params.lora_name:
            query = query.where(
                Image.loras.op("@>")(f'[{{"name": "{params.lora_name}"}}]')
            )

        # Full-text search in prompts
        if params.q:
            search_terms = params.q.replace(" ", " & ")
            query = query.where(
                func.to_tsvector("english", func.coalesce(Image.positive_prompt, "")).op("@@")(
                    func.to_tsquery("english", search_terms)
                )
            )

        # Get total count
        count_query = select(func.count()).select_from(query.subquery())
        total_result = await self.db.execute(count_query)
        total = total_result.scalar() or 0

        # Apply sorting
        sort_column = getattr(Image, params.sort_by, Image.created_at)
        if params.sort_order == "asc":
            query = query.order_by(sort_column.asc())
        else:
            query = query.order_by(sort_column.desc())

        # Apply pagination
        offset = (params.page - 1) * params.per_page
        query = query.offset(offset).limit(params.per_page)

        # Execute query
        result = await self.db.execute(query)
        images = result.scalars().all()

        # Convert to response model
        items = [ImageListResponse.model_validate(img) for img in images]
        total_pages = (total + params.per_page - 1) // params.per_page

        return PaginatedResponse(
            items=items,
            total=total,
            page=params.page,
            per_page=params.per_page,
            total_pages=total_pages,
        )

    async def get_image(self, image_id: UUID) -> ImageResponse | None:
        """Get a single image by ID."""
        result = await self.db.execute(select(Image).where(Image.id == image_id))
        image = result.scalar_one_or_none()

        if not image:
            return None

        return ImageResponse.model_validate(image)

    async def update_image(
        self, image_id: UUID, update_data: ImageUpdate
    ) -> ImageResponse | None:
        """Update image metadata."""
        result = await self.db.execute(select(Image).where(Image.id == image_id))
        image = result.scalar_one_or_none()

        if not image:
            return None

        update_dict = update_data.model_dump(exclude_unset=True)
        for key, value in update_dict.items():
            setattr(image, key, value)

        await self.db.commit()
        await self.db.refresh(image)

        return ImageResponse.model_validate(image)

    async def delete_image(self, image_id: UUID, permanent: bool = False) -> bool:
        """Delete an image (soft or permanent)."""
        result = await self.db.execute(select(Image).where(Image.id == image_id))
        image = result.scalar_one_or_none()

        if not image:
            return False

        if permanent:
            await self.db.delete(image)
        else:
            image.deleted_at = datetime.utcnow()

        await self.db.commit()
        return True

    async def restore_image(self, image_id: UUID) -> bool:
        """Restore a soft-deleted image."""
        result = await self.db.execute(
            select(Image).where(Image.id == image_id, Image.deleted_at.is_not(None))
        )
        image = result.scalar_one_or_none()

        if not image:
            return False

        image.deleted_at = None
        await self.db.commit()
        return True
