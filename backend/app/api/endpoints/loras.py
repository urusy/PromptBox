"""API endpoints for LoRA resources."""

import re
from typing import Literal

from fastapi import APIRouter, Query
from loguru import logger
from sqlalchemy import Numeric, case, func, select

from app.api.deps import CurrentUser, DbSession
from app.models.image import Image
from app.schemas.model import (
    CivitaiInfoResponse,
    LoraDetail,
    LoraListItem,
    LoraListResponse,
)
from app.services.civitai_service import civitai_service

router = APIRouter(prefix="/loras", tags=["loras"])


def extract_display_name(full_name: str) -> str:
    """Extract display name from full path.

    Examples:
        loras\\character\\some_lora.safetensors -> some_lora.safetensors
        /path/to/lora.ckpt -> lora.ckpt
        lora_name -> lora_name
    """
    name = full_name.replace("\\", "/")
    if "/" in name:
        name = name.rsplit("/", 1)[-1]
    return name


@router.get("", response_model=LoraListResponse)
async def get_loras(
    db: DbSession,
    _: CurrentUser,
    q: str | None = Query(None, description="Search query for LoRA name"),
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
) -> LoraListResponse:
    """Get list of LoRAs with usage statistics."""
    logger.debug(f"Getting LoRAs list: q={q}, min_count={min_count}")

    base_filter = Image.deleted_at.is_(None)

    # Unnest JSONB loras array to get individual LoRA info
    # Include hash for later CivitAI lookup
    lora_unnest = (
        select(
            Image.id,
            Image.rating,
            func.jsonb_array_elements(Image.loras).label("lora_obj"),
        )
        .where(base_filter)
        .where(func.jsonb_array_length(Image.loras) > 0)
        .subquery()
    )

    # Extract name and hash from JSONB object
    lora_data = select(
        lora_unnest.c.id,
        lora_unnest.c.rating,
        lora_unnest.c.lora_obj.op("->>")("name").label("lora_name"),
        lora_unnest.c.lora_obj.op("->>")("hash").label("lora_hash"),
    ).subquery()

    # Build aggregation query
    query = (
        select(
            lora_data.c.lora_name,
            func.max(lora_data.c.lora_hash).label("hash"),  # Take any non-null hash
            func.count().label("image_count"),
            func.count(case((lora_data.c.rating > 0, 1))).label("rated_count"),
            func.avg(case((lora_data.c.rating > 0, lora_data.c.rating))).label(
                "avg_rating"
            ),
            func.count(case((lora_data.c.rating >= 4, 1))).label("high_rated_count"),
        )
        .group_by(lora_data.c.lora_name)
        .having(func.count() >= min_count)
    )

    # Apply filters
    if q:
        query = query.having(func.lower(lora_data.c.lora_name).contains(q.lower()))

    if min_rating is not None:
        query = query.having(
            func.avg(case((lora_data.c.rating > 0, lora_data.c.rating))) >= min_rating
        )

    # Apply sorting
    if sort_by == "count":
        order_col = func.count()
    elif sort_by == "rating":
        order_col = func.avg(case((lora_data.c.rating > 0, lora_data.c.rating)))
    else:  # name
        order_col = lora_data.c.lora_name

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
        LoraListItem(
            name=row.lora_name,
            display_name=extract_display_name(row.lora_name),
            hash=row.hash,
            image_count=row.image_count,
            rated_count=row.rated_count,
            avg_rating=round(float(row.avg_rating), 2) if row.avg_rating else None,
            high_rated_count=row.high_rated_count,
        )
        for row in rows
    ]

    return LoraListResponse(items=items, total=total)


@router.get("/{lora_name:path}/detail", response_model=LoraDetail)
async def get_lora_detail(
    db: DbSession,
    _: CurrentUser,
    lora_name: str,
) -> LoraDetail:
    """Get detailed statistics for a specific LoRA."""
    logger.debug(f"Getting LoRA detail: {lora_name}")

    base_filter = Image.deleted_at.is_(None)

    # Unnest and filter for this specific LoRA
    lora_unnest = (
        select(
            Image.id,
            Image.rating,
            Image.model_name,
            Image.sampler_name,
            func.jsonb_array_elements(Image.loras).label("lora_obj"),
        )
        .where(base_filter)
        .where(func.jsonb_array_length(Image.loras) > 0)
        .subquery()
    )

    lora_filtered = (
        select(
            lora_unnest.c.id,
            lora_unnest.c.rating,
            lora_unnest.c.model_name,
            lora_unnest.c.sampler_name,
            lora_unnest.c.lora_obj.op("->>")("name").label("lora_name"),
            lora_unnest.c.lora_obj.op("->>")("hash").label("lora_hash"),
            lora_unnest.c.lora_obj.op("->>")("weight")
            .cast(Numeric)
            .label("lora_weight"),
        )
        .where(lora_unnest.c.lora_obj.op("->>")("name") == lora_name)
        .subquery()
    )

    # Basic stats
    stats_result = await db.execute(
        select(
            func.max(lora_filtered.c.lora_hash).label("hash"),
            func.count().label("image_count"),
            func.count(case((lora_filtered.c.rating > 0, 1))).label("rated_count"),
            func.avg(case((lora_filtered.c.rating > 0, lora_filtered.c.rating))).label(
                "avg_rating"
            ),
            func.count(case((lora_filtered.c.rating >= 4, 1))).label(
                "high_rated_count"
            ),
            func.avg(lora_filtered.c.lora_weight).label("avg_weight"),
        )
    )
    stats_row = stats_result.first()

    if not stats_row or stats_row.image_count == 0:
        return LoraDetail(
            name=lora_name,
            display_name=extract_display_name(lora_name),
            hash=None,
            image_count=0,
            rated_count=0,
            avg_rating=None,
            high_rated_count=0,
            rating_distribution=dict.fromkeys(range(6), 0),
            avg_weight=None,
            top_models=[],
            top_samplers=[],
        )

    # Rating distribution
    rating_dist_result = await db.execute(
        select(lora_filtered.c.rating, func.count().label("count")).group_by(
            lora_filtered.c.rating
        )
    )
    rating_distribution = dict.fromkeys(range(6), 0)
    for row in rating_dist_result.all():
        rating_distribution[row.rating] = row.count

    # Top models
    model_result = await db.execute(
        select(
            lora_filtered.c.model_name,
            func.count().label("count"),
            func.avg(case((lora_filtered.c.rating > 0, lora_filtered.c.rating))).label(
                "avg_rating"
            ),
        )
        .where(lora_filtered.c.model_name.isnot(None))
        .group_by(lora_filtered.c.model_name)
        .order_by(func.count().desc())
        .limit(10)
    )
    top_models = [
        {
            "name": row.model_name,
            "count": row.count,
            "avg_rating": round(float(row.avg_rating), 2) if row.avg_rating else None,
        }
        for row in model_result.all()
    ]

    # Top samplers
    sampler_result = await db.execute(
        select(
            lora_filtered.c.sampler_name,
            func.count().label("count"),
            func.avg(case((lora_filtered.c.rating > 0, lora_filtered.c.rating))).label(
                "avg_rating"
            ),
        )
        .where(lora_filtered.c.sampler_name.isnot(None))
        .group_by(lora_filtered.c.sampler_name)
        .order_by(func.count().desc())
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

    return LoraDetail(
        name=lora_name,
        display_name=extract_display_name(lora_name),
        hash=stats_row.hash,
        image_count=stats_row.image_count,
        rated_count=stats_row.rated_count,
        avg_rating=(
            round(float(stats_row.avg_rating), 2) if stats_row.avg_rating else None
        ),
        high_rated_count=stats_row.high_rated_count,
        rating_distribution=rating_distribution,
        avg_weight=(
            round(float(stats_row.avg_weight), 2) if stats_row.avg_weight else None
        ),
        top_models=top_models,
        top_samplers=top_samplers,
    )


@router.get("/{lora_name:path}/civitai", response_model=CivitaiInfoResponse)
async def get_lora_civitai_info(
    db: DbSession,
    _: CurrentUser,
    lora_name: str,
) -> CivitaiInfoResponse:
    """Get CivitAI information for a LoRA.

    First tries to find by hash (more accurate), then falls back to name search.
    """
    logger.debug(f"Getting CivitAI info for LoRA: {lora_name}")

    base_filter = Image.deleted_at.is_(None)

    # Try to get hash for this LoRA
    lora_unnest = (
        select(
            func.jsonb_array_elements(Image.loras).label("lora_obj"),
        )
        .where(base_filter)
        .where(func.jsonb_array_length(Image.loras) > 0)
        .subquery()
    )

    hash_result = await db.execute(
        select(
            lora_unnest.c.lora_obj.op("->>")("hash").label("hash"),
        )
        .where(lora_unnest.c.lora_obj.op("->>")("name") == lora_name)
        .where(lora_unnest.c.lora_obj.op("->>")("hash").isnot(None))
        .limit(1)
    )
    hash_row = hash_result.first()
    lora_hash = hash_row.hash if hash_row else None

    # Try hash lookup first (more accurate)
    if lora_hash:
        info = await civitai_service.get_model_by_hash(lora_hash)
        if info:
            return CivitaiInfoResponse(found=True, info=info)

    # Fall back to name search
    display_name = extract_display_name(lora_name)
    search_name = re.sub(
        r"\.(safetensors|ckpt|pt)$", "", display_name, flags=re.IGNORECASE
    )

    info = await civitai_service.get_model_info(search_name, model_type="LORA")

    if info:
        return CivitaiInfoResponse(found=True, info=info)
    else:
        return CivitaiInfoResponse(found=False, error="LoRA not found on CivitAI")
