from datetime import datetime
from uuid import UUID

from sqlalchemy import select, update
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.image import Image


class BatchService:
    def __init__(self, db: AsyncSession) -> None:
        self.db = db

    async def batch_update(
        self,
        ids: list[UUID],
        rating: int | None = None,
        is_favorite: bool | None = None,
        needs_improvement: bool | None = None,
        add_tags: list[str] | None = None,
        remove_tags: list[str] | None = None,
    ) -> int:
        """Update multiple images at once."""
        # Build update values
        values: dict = {}

        if rating is not None:
            values["rating"] = rating
        if is_favorite is not None:
            values["is_favorite"] = is_favorite
        if needs_improvement is not None:
            values["needs_improvement"] = needs_improvement

        # For tag operations, we need to handle each image individually
        if add_tags or remove_tags:
            return await self._batch_update_tags(ids, add_tags, remove_tags, values)

        if not values:
            return 0

        # Simple batch update without tag changes
        stmt = (
            update(Image)
            .where(Image.id.in_(ids))
            .where(Image.deleted_at.is_(None))
            .values(**values)
        )
        result = await self.db.execute(stmt)
        await self.db.commit()

        return result.rowcount

    async def _batch_update_tags(
        self,
        ids: list[UUID],
        add_tags: list[str] | None,
        remove_tags: list[str] | None,
        other_values: dict,
    ) -> int:
        """Update tags for multiple images."""
        result = await self.db.execute(
            select(Image).where(Image.id.in_(ids)).where(Image.deleted_at.is_(None))
        )
        images = result.scalars().all()

        count = 0
        for image in images:
            # Update tags
            current_tags = set(image.user_tags or [])

            if remove_tags:
                current_tags -= set(remove_tags)
            if add_tags:
                current_tags |= set(add_tags)

            image.user_tags = list(current_tags)

            # Apply other updates
            for key, value in other_values.items():
                setattr(image, key, value)

            count += 1

        await self.db.commit()
        return count

    async def batch_delete(self, ids: list[UUID], permanent: bool = False) -> int:
        """Delete multiple images."""
        if permanent:
            # Get images to delete files
            result = await self.db.execute(
                select(Image).where(Image.id.in_(ids))
            )
            images = result.scalars().all()

            for image in images:
                await self.db.delete(image)

            await self.db.commit()
            return len(images)
        else:
            # Soft delete
            stmt = (
                update(Image)
                .where(Image.id.in_(ids))
                .where(Image.deleted_at.is_(None))
                .values(deleted_at=datetime.utcnow())
            )
            result = await self.db.execute(stmt)
            await self.db.commit()
            return result.rowcount

    async def batch_restore(self, ids: list[UUID]) -> int:
        """Restore multiple deleted images."""
        stmt = (
            update(Image)
            .where(Image.id.in_(ids))
            .where(Image.deleted_at.is_not(None))
            .values(deleted_at=None)
        )
        result = await self.db.execute(stmt)
        await self.db.commit()
        return result.rowcount
