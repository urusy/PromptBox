from typing import Any
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.image import Image


class ExportService:
    def __init__(self, db: AsyncSession) -> None:
        self.db = db

    async def get_export_data(self, ids: list[UUID] | None = None) -> list[dict[str, Any]]:
        """Get image data for export."""
        query = select(Image).where(Image.deleted_at.is_(None))

        if ids:
            query = query.where(Image.id.in_(ids))

        query = query.order_by(Image.created_at.desc())

        result = await self.db.execute(query)
        images = result.scalars().all()

        data = []
        for img in images:
            data.append({
                "id": str(img.id),
                "original_filename": img.original_filename,
                "source_tool": img.source_tool,
                "model_type": img.model_type,
                "model_name": img.model_name,
                "positive_prompt": img.positive_prompt,
                "negative_prompt": img.negative_prompt,
                "sampler_name": img.sampler_name,
                "scheduler": img.scheduler,
                "steps": img.steps,
                "cfg_scale": float(img.cfg_scale) if img.cfg_scale else None,
                "seed": img.seed,
                "width": img.width,
                "height": img.height,
                "rating": img.rating,
                "is_favorite": img.is_favorite,
                "user_tags": ",".join(img.user_tags) if img.user_tags else "",
                "user_memo": img.user_memo,
                "created_at": img.created_at.isoformat(),
            })

        return data

    async def get_prompts_export(self, ids: list[UUID] | None = None) -> str:
        """Get prompts for export as text."""
        query = select(Image).where(Image.deleted_at.is_(None))

        if ids:
            query = query.where(Image.id.in_(ids))

        query = query.order_by(Image.created_at.desc())

        result = await self.db.execute(query)
        images = result.scalars().all()

        lines = []
        for img in images:
            lines.append(f"=== {img.original_filename} ===")
            lines.append(f"Model: {img.model_name or 'Unknown'}")

            if img.positive_prompt:
                lines.append(f"\nPositive Prompt:\n{img.positive_prompt}")

            if img.negative_prompt:
                lines.append(f"\nNegative Prompt:\n{img.negative_prompt}")

            if img.loras:
                lora_strs = [f"{l.get('name', 'Unknown')}:{l.get('weight', 1.0)}" for l in img.loras]
                lines.append(f"\nLoRAs: {', '.join(lora_strs)}")

            lines.append(f"\nSettings: Steps={img.steps}, CFG={img.cfg_scale}, Sampler={img.sampler_name}, Seed={img.seed}")
            lines.append("\n" + "-" * 50 + "\n")

        return "\n".join(lines)
