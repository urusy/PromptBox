"""API endpoints for Model resources."""

import re
from typing import Literal

from fastapi import APIRouter, Query
from loguru import logger
from sqlalchemy import case, func, select

from app.api.deps import CurrentUser, DbSession
from app.models.image import Image
from app.schemas.model import (
    CivitaiInfoResponse,
    ModelDetail,
    ModelListItem,
    ModelListResponse,
)
from app.services.civitai_service import civitai_service

router = APIRouter(prefix="/models", tags=["models"])


def extract_display_name(full_name: str) -> str:
    """Extract display name from full path.

    Examples:
        models\\checkpoints\\animagine.safetensors -> animagine.safetensors
        /path/to/model.ckpt -> model.ckpt
        model_name -> model_name
    """
    # Handle both Windows and Unix path separators
    name = full_name.replace("\\", "/")
    if "/" in name:
        name = name.rsplit("/", 1)[-1]
    # Remove common extensions for cleaner display (optional)
    # name = re.sub(r'\.(safetensors|ckpt|pt)$', '', name, flags=re.IGNORECASE)
    return name


@router.get("", response_model=ModelListResponse)
async def get_models(
    db: DbSession,
    _: CurrentUser,
    q: str | None = Query(None, description="Search query for model name"),
    model_type: str | None = Query(None, description="Filter by model type"),
    min_count: int = Query(1, ge=1, description="Minimum image count"),
    min_rating: float | None = Query(
        None, ge=0, le=5, description="Minimum average rating"
    ),
    sort_by: Literal["count", "rating", "name"] = Query(
        "count", description="Sort field"
    ),
    sort_order: Literal["asc", "desc"] = Query("desc", description="Sort order"),
    limit: int = Query(100, ge=1, le=500, description="Maximum results"),
    offset: int = Query(0, ge=0, description="Offset for pagination"),
) -> ModelListResponse:
    """Get list of models with usage statistics."""
    logger.debug(
        f"Getting models list: q={q}, model_type={model_type}, min_count={min_count}"
    )

    base_filter = Image.deleted_at.is_(None)

    # Build query for model statistics
    query = (
        select(
            Image.model_name,
            Image.model_type,
            func.count(Image.id).label("image_count"),
            func.count(case((Image.rating > 0, 1))).label("rated_count"),
            func.avg(case((Image.rating > 0, Image.rating))).label("avg_rating"),
            func.count(case((Image.rating >= 4, 1))).label("high_rated_count"),
        )
        .where(base_filter)
        .where(Image.model_name.isnot(None))
        .group_by(Image.model_name, Image.model_type)
        .having(func.count(Image.id) >= min_count)
    )

    # Apply filters
    if q:
        query = query.having(func.lower(Image.model_name).contains(q.lower()))

    if model_type:
        query = query.having(Image.model_type == model_type)

    if min_rating is not None:
        query = query.having(
            func.avg(case((Image.rating > 0, Image.rating))) >= min_rating
        )

    # Apply sorting
    if sort_by == "count":
        order_col = func.count(Image.id)
    elif sort_by == "rating":
        order_col = func.avg(case((Image.rating > 0, Image.rating)))
    else:  # name
        order_col = Image.model_name

    if sort_order == "desc":
        query = query.order_by(order_col.desc().nulls_last())
    else:
        query = query.order_by(order_col.asc().nulls_last())

    # Get total count first
    count_query = select(func.count()).select_from(query.subquery())
    total_result = await db.execute(count_query)
    total = total_result.scalar() or 0

    # Apply pagination
    query = query.limit(limit).offset(offset)

    result = await db.execute(query)
    rows = result.all()

    items = [
        ModelListItem(
            name=row.model_name,
            display_name=extract_display_name(row.model_name),
            model_type=row.model_type,
            image_count=row.image_count,
            rated_count=row.rated_count,
            avg_rating=round(float(row.avg_rating), 2) if row.avg_rating else None,
            high_rated_count=row.high_rated_count,
        )
        for row in rows
    ]

    return ModelListResponse(items=items, total=total)


@router.get("/{model_name:path}/detail", response_model=ModelDetail)
async def get_model_detail(
    db: DbSession,
    _: CurrentUser,
    model_name: str,
) -> ModelDetail:
    """Get detailed statistics for a specific model."""
    logger.debug(f"Getting model detail: {model_name}")

    base_filter = Image.deleted_at.is_(None)
    model_filter = Image.model_name == model_name

    # Basic stats
    stats_result = await db.execute(
        select(
            Image.model_type,
            func.count(Image.id).label("image_count"),
            func.count(case((Image.rating > 0, 1))).label("rated_count"),
            func.avg(case((Image.rating > 0, Image.rating))).label("avg_rating"),
            func.count(case((Image.rating >= 4, 1))).label("high_rated_count"),
        )
        .where(base_filter)
        .where(model_filter)
        .group_by(Image.model_type)
    )
    stats_row = stats_result.first()

    if not stats_row:
        # Return empty stats if model not found
        return ModelDetail(
            name=model_name,
            display_name=extract_display_name(model_name),
            model_type=None,
            image_count=0,
            rated_count=0,
            avg_rating=None,
            high_rated_count=0,
            rating_distribution=dict.fromkeys(range(6), 0),
            top_samplers=[],
            top_loras=[],
        )

    # Rating distribution
    rating_dist_result = await db.execute(
        select(Image.rating, func.count(Image.id).label("count"))
        .where(base_filter)
        .where(model_filter)
        .group_by(Image.rating)
    )
    rating_distribution = dict.fromkeys(range(6), 0)
    for row in rating_dist_result.all():
        rating_distribution[row.rating] = row.count

    # Top samplers
    sampler_result = await db.execute(
        select(
            Image.sampler_name,
            func.count(Image.id).label("count"),
            func.avg(case((Image.rating > 0, Image.rating))).label("avg_rating"),
        )
        .where(base_filter)
        .where(model_filter)
        .where(Image.sampler_name.isnot(None))
        .group_by(Image.sampler_name)
        .order_by(func.count(Image.id).desc())
        .limit(10)
    )
    top_samplers = [
        {
            "name": row.sampler_name,
            "count": row.count,
            "avg_rating": round(float(row.avg_rating), 2) if row.avg_rating else None,
        }
        for row in sampler_result.all()
    ]

    # Top LoRAs (unnest JSONB)
    lora_unnest = (
        select(
            Image.id,
            Image.rating,
            func.jsonb_array_elements(Image.loras).op("->>")("name").label("lora_name"),
        )
        .where(base_filter)
        .where(model_filter)
        .where(func.jsonb_array_length(Image.loras) > 0)
        .subquery()
    )
    lora_result = await db.execute(
        select(
            lora_unnest.c.lora_name,
            func.count().label("count"),
            func.avg(case((lora_unnest.c.rating > 0, lora_unnest.c.rating))).label(
                "avg_rating"
            ),
        )
        .group_by(lora_unnest.c.lora_name)
        .order_by(func.count().desc())
        .limit(10)
    )
    top_loras = [
        {
            "name": row.lora_name,
            "count": row.count,
            "avg_rating": round(float(row.avg_rating), 2) if row.avg_rating else None,
        }
        for row in lora_result.all()
    ]

    return ModelDetail(
        name=model_name,
        display_name=extract_display_name(model_name),
        model_type=stats_row.model_type,
        image_count=stats_row.image_count,
        rated_count=stats_row.rated_count,
        avg_rating=(
            round(float(stats_row.avg_rating), 2) if stats_row.avg_rating else None
        ),
        high_rated_count=stats_row.high_rated_count,
        rating_distribution=rating_distribution,
        top_samplers=top_samplers,
        top_loras=top_loras,
    )


@router.get("/{model_name:path}/civitai", response_model=CivitaiInfoResponse)
async def get_model_civitai_info(
    _: CurrentUser,
    model_name: str,
) -> CivitaiInfoResponse:
    """Get CivitAI information for a model."""
    logger.debug(f"Getting CivitAI info for model: {model_name}")

    display_name = extract_display_name(model_name)
    # Remove extension for search
    search_name = re.sub(
        r"\.(safetensors|ckpt|pt)$", "", display_name, flags=re.IGNORECASE
    )

    info = await civitai_service.get_model_info(search_name, model_type="Checkpoint")

    if info:
        return CivitaiInfoResponse(found=True, info=info)
    else:
        return CivitaiInfoResponse(found=False, error="Model not found on CivitAI")
