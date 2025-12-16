import asyncio
import re
from datetime import datetime
from uuid import UUID

from sqlalchemy import Select, func, select, type_coerce
from sqlalchemy.dialects.postgresql import JSONB
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.image import Image
from app.schemas.common import PaginatedResponse
from app.schemas.image import (
    ImageListResponse,
    ImageResponse,
    ImageSearchParams,
    ImageUpdate,
)


def escape_like_pattern(value: str) -> str:
    """Escape special characters in LIKE patterns (%, _, \\)."""
    return re.sub(r"([%_\\])", r"\\\1", value)


class ImageService:
    def __init__(self, db: AsyncSession) -> None:
        self.db = db

    def _build_search_query(self, params: ImageSearchParams) -> Select[tuple[Image]]:
        """Build search query with filters (without sorting/pagination)."""
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
            escaped_name = escape_like_pattern(params.model_name)
            query = query.where(
                Image.model_name.ilike(f"%{escaped_name}%", escape="\\")
            )

        if params.sampler_name:
            query = query.where(Image.sampler_name == params.sampler_name)

        if params.exact_rating is not None:
            query = query.where(Image.rating == params.exact_rating)
        elif params.min_rating is not None:
            query = query.where(Image.rating >= params.min_rating)

        if params.is_favorite is not None:
            query = query.where(Image.is_favorite == params.is_favorite)

        if params.needs_improvement is not None:
            query = query.where(Image.needs_improvement == params.needs_improvement)

        if params.tags:
            for tag in params.tags:
                query = query.where(Image.user_tags.contains([tag]))

        if params.lora_name:
            # Use type_coerce to properly pass Python dict as JSONB
            lora_filter = [{"name": params.lora_name}]
            query = query.where(Image.loras.op("@>")(type_coerce(lora_filter, JSONB)))

        # Filter by XYZ grid
        if params.is_xyz_grid is not None:
            if params.is_xyz_grid:
                # Grid images only (is_xyz_grid = true in model_params)
                query = query.where(
                    Image.model_params.op("->>")("is_xyz_grid") == "true"
                )
            else:
                # Non-grid images only (is_xyz_grid is null or false)
                query = query.where(
                    (Image.model_params.op("->>")("is_xyz_grid").is_(None))
                    | (Image.model_params.op("->>")("is_xyz_grid") != "true")
                )

        # Filter by upscaled (hires_upscaler exists in model_params)
        if params.is_upscaled is not None:
            if params.is_upscaled:
                # Upscaled images only
                query = query.where(
                    Image.model_params.op("->>")("hires_upscaler").is_not(None)
                )
            else:
                # Non-upscaled images only
                query = query.where(
                    Image.model_params.op("->>")("hires_upscaler").is_(None)
                )

        # Filter by orientation (portrait, landscape, square)
        if params.orientation:
            if params.orientation == "portrait":
                query = query.where(Image.height > Image.width)
            elif params.orientation == "landscape":
                query = query.where(Image.width > Image.height)
            elif params.orientation == "square":
                query = query.where(Image.width == Image.height)

        # Filter by minimum dimensions
        if params.min_width is not None:
            query = query.where(Image.width >= params.min_width)

        if params.min_height is not None:
            query = query.where(Image.height >= params.min_height)

        # Filter by date
        if params.date_from:
            try:
                date_from = datetime.fromisoformat(params.date_from)
                query = query.where(Image.created_at >= date_from)
            except ValueError:
                pass  # Ignore invalid date format

        # Filter by seed (exact match or range with tolerance)
        if params.seed is not None:
            if params.seed_tolerance is not None and params.seed_tolerance > 0:
                # Search seeds within +/- tolerance
                min_seed = params.seed - params.seed_tolerance
                max_seed = params.seed + params.seed_tolerance
                query = query.where(
                    Image.seed.is_not(None),
                    Image.seed >= min_seed,
                    Image.seed <= max_seed,
                )
            else:
                # Exact seed match
                query = query.where(Image.seed == params.seed)

        # Full-text search in prompts
        if params.q:
            search_terms = params.q.replace(" ", " & ")
            query = query.where(
                func.to_tsvector(
                    "english", func.coalesce(Image.positive_prompt, "")
                ).op("@@")(func.to_tsquery("english", search_terms))
            )

        return query

    async def list_images(
        self, params: ImageSearchParams
    ) -> PaginatedResponse[ImageListResponse]:
        """List images with search and pagination."""
        query = self._build_search_query(params)

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

    async def get_image_with_neighbors(
        self, image_id: UUID, search_params: ImageSearchParams | None = None
    ) -> ImageResponse | None:
        """Get a single image by ID with prev/next IDs based on search context.

        Optimized to run prev/next queries in parallel for better performance.
        """
        result = await self.db.execute(select(Image).where(Image.id == image_id))
        image = result.scalar_one_or_none()

        if not image:
            return None

        response = ImageResponse.model_validate(image)

        # If no search params, return without neighbors
        if not search_params:
            return response

        # Build the same query used for list
        base_query = self._build_search_query(search_params)

        # Get the sort column and order
        sort_column = getattr(Image, search_params.sort_by, Image.created_at)
        current_sort_value = getattr(image, search_params.sort_by, image.created_at)

        # Build prev/next queries
        # Find prev image (the one that comes before in the sorted order)
        # For desc order: prev has greater sort value
        # For asc order: prev has lesser sort value
        if search_params.sort_order == "asc":
            prev_query = (
                base_query.where(
                    (sort_column < current_sort_value)
                    | ((sort_column == current_sort_value) & (Image.id < image_id))
                )
                .order_by(sort_column.desc(), Image.id.desc())
                .limit(1)
            )
        else:
            prev_query = (
                base_query.where(
                    (sort_column > current_sort_value)
                    | ((sort_column == current_sort_value) & (Image.id > image_id))
                )
                .order_by(sort_column.asc(), Image.id.asc())
                .limit(1)
            )

        # Find next image (the one that comes after in the sorted order)
        # For desc order: next has lesser sort value
        # For asc order: next has greater sort value
        if search_params.sort_order == "asc":
            next_query = (
                base_query.where(
                    (sort_column > current_sort_value)
                    | ((sort_column == current_sort_value) & (Image.id > image_id))
                )
                .order_by(sort_column.asc(), Image.id.asc())
                .limit(1)
            )
        else:
            next_query = (
                base_query.where(
                    (sort_column < current_sort_value)
                    | ((sort_column == current_sort_value) & (Image.id < image_id))
                )
                .order_by(sort_column.desc(), Image.id.desc())
                .limit(1)
            )

        # Execute prev/next queries in parallel
        prev_result, next_result = await asyncio.gather(
            self.db.execute(prev_query),
            self.db.execute(next_query),
        )

        prev_image = prev_result.scalar_one_or_none()
        if prev_image:
            response.prev_id = prev_image.id

        next_image = next_result.scalar_one_or_none()
        if next_image:
            response.next_id = next_image.id

        return response

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
