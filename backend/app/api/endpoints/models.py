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
    ModelVersionStats,
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


def extract_base_model_name(display_name: str) -> str:
    """Extract base model name by removing version suffixes.

    Examples:
        animagine_v80.safetensors -> animagine.safetensors
        cyberIllustrious_v50.safetensors -> cyberIllustrious.safetensors
        model_V2.0.ckpt -> model.ckpt
        pony_v6.safetensors -> pony.safetensors
        noobaiXL_v1.1.safetensors -> noobaiXL.safetensors
    """
    # Remove extension first, process, then add back
    ext_match = re.search(r"\.(safetensors|ckpt|pt)$", display_name, re.IGNORECASE)
    ext = ext_match.group(0) if ext_match else ""
    name_without_ext = display_name[: -len(ext)] if ext else display_name

    # Remove version patterns:
    # - _v80, _V2, _v1.5, _v50VAE, etc.
    # - v6, V2.0 at the end
    # - -v1, -V2.0, etc.
    base_name = re.sub(
        r"[_-]?[vV]\d+(\.\d+)?[a-zA-Z]*$",
        "",
        name_without_ext,
    )

    # If nothing left after removing version, keep original
    if not base_name:
        base_name = name_without_ext

    return base_name + ext


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
    """Get list of models with usage statistics, grouped by base model name."""
    logger.debug(
        f"Getting models list: q={q}, model_type={model_type}, min_count={min_count}"
    )

    base_filter = Image.deleted_at.is_(None)

    # Build query for per-model statistics
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
    )

    # Apply model_type filter at SQL level
    if model_type:
        query = query.having(Image.model_type == model_type)

    result = await db.execute(query)
    rows = result.all()

    # Group by base model name in Python
    from collections import defaultdict

    base_model_groups: dict[str, dict] = defaultdict(
        lambda: {
            "versions": [],
            "model_type": None,
            "image_count": 0,
            "rated_count": 0,
            "rating_sum": 0.0,
            "rating_count": 0,
            "high_rated_count": 0,
        }
    )

    for row in rows:
        display_name = extract_display_name(row.model_name)
        base_name = extract_base_model_name(display_name)

        group = base_model_groups[base_name]
        group["versions"].append(row.model_name)
        group["model_type"] = group["model_type"] or row.model_type
        group["image_count"] += row.image_count
        group["rated_count"] += row.rated_count
        group["high_rated_count"] += row.high_rated_count
        if row.avg_rating is not None:
            group["rating_sum"] += float(row.avg_rating) * row.rated_count
            group["rating_count"] += row.rated_count

    # Convert to list and apply filters
    grouped_models = []
    for base_name, group in base_model_groups.items():
        avg_rating = (
            group["rating_sum"] / group["rating_count"]
            if group["rating_count"] > 0
            else None
        )

        # Apply min_count filter
        if group["image_count"] < min_count:
            continue

        # Apply min_rating filter
        if min_rating is not None and (avg_rating is None or avg_rating < min_rating):
            continue

        # Apply search filter
        if q and q.lower() not in base_name.lower():
            continue

        grouped_models.append(
            {
                "base_name": base_name,
                "model_type": group["model_type"],
                "image_count": group["image_count"],
                "rated_count": group["rated_count"],
                "avg_rating": round(avg_rating, 2) if avg_rating else None,
                "high_rated_count": group["high_rated_count"],
                "version_count": len(group["versions"]),
            }
        )

    # Sort
    reverse = sort_order == "desc"
    if sort_by == "count":
        grouped_models.sort(key=lambda x: x["image_count"], reverse=reverse)
    elif sort_by == "rating":
        grouped_models.sort(
            key=lambda x: (x["avg_rating"] or 0, x["image_count"]), reverse=reverse
        )
    else:  # name
        grouped_models.sort(key=lambda x: x["base_name"].lower(), reverse=reverse)

    # Get total and apply pagination
    total = len(grouped_models)
    paginated = grouped_models[offset : offset + limit]

    items = [
        ModelListItem(
            name=m["base_name"],
            display_name=m["base_name"],
            model_type=m["model_type"],
            image_count=m["image_count"],
            rated_count=m["rated_count"],
            avg_rating=m["avg_rating"],
            high_rated_count=m["high_rated_count"],
            version_count=m["version_count"],
        )
        for m in paginated
    ]

    return ModelListResponse(items=items, total=total)


@router.get("/{model_name:path}/detail", response_model=ModelDetail)
async def get_model_detail(
    db: DbSession,
    _: CurrentUser,
    model_name: str,
) -> ModelDetail:
    """Get detailed statistics for a specific model (base name, version removed).

    The model_name parameter should be the base model name without version suffix.
    This endpoint returns aggregated stats across all versions plus per-version stats.
    """
    logger.debug(f"Getting model detail: {model_name}")

    base_filter = Image.deleted_at.is_(None)

    # First, get all distinct model names from DB
    all_models_result = await db.execute(
        select(Image.model_name).where(base_filter).where(Image.model_name.isnot(None)).distinct()
    )
    all_model_names = [row[0] for row in all_models_result.all()]

    # Find all versions that match this base model name
    matching_versions = []
    for full_name in all_model_names:
        display_name = extract_display_name(full_name)
        base_name = extract_base_model_name(display_name)
        if base_name == model_name:
            matching_versions.append(full_name)

    if not matching_versions:
        # Return empty stats if model not found
        return ModelDetail(
            name=model_name,
            display_name=model_name,
            model_type=None,
            image_count=0,
            rated_count=0,
            avg_rating=None,
            high_rated_count=0,
            rating_distribution=dict.fromkeys(range(6), 0),
            top_samplers=[],
            top_loras=[],
            versions=[],
        )

    # Create filter for all matching versions
    from sqlalchemy import or_

    versions_filter = or_(*[Image.model_name == v for v in matching_versions])

    # Get per-version statistics
    version_stats_result = await db.execute(
        select(
            Image.model_name,
            Image.model_type,
            func.count(Image.id).label("image_count"),
            func.count(case((Image.rating > 0, 1))).label("rated_count"),
            func.avg(case((Image.rating > 0, Image.rating))).label("avg_rating"),
            func.count(case((Image.rating >= 4, 1))).label("high_rated_count"),
        )
        .where(base_filter)
        .where(versions_filter)
        .group_by(Image.model_name, Image.model_type)
        .order_by(func.count(Image.id).desc())
    )
    version_rows = version_stats_result.all()

    # Get per-version rating distributions
    version_rating_dist: dict[str, dict[int, int]] = {}
    for v_name in matching_versions:
        dist_result = await db.execute(
            select(Image.rating, func.count(Image.id).label("count"))
            .where(base_filter)
            .where(Image.model_name == v_name)
            .group_by(Image.rating)
        )
        dist = dict.fromkeys(range(6), 0)
        for row in dist_result.all():
            dist[row.rating] = row.count
        version_rating_dist[v_name] = dist

    # Build version stats list
    versions: list[ModelVersionStats] = []
    model_type = None
    for row in version_rows:
        display_name = extract_display_name(row.model_name)
        model_type = model_type or row.model_type
        versions.append(
            ModelVersionStats(
                name=row.model_name,
                display_name=display_name,
                image_count=row.image_count,
                rated_count=row.rated_count,
                avg_rating=round(float(row.avg_rating), 2) if row.avg_rating else None,
                high_rated_count=row.high_rated_count,
                rating_distribution=version_rating_dist.get(
                    row.model_name, dict.fromkeys(range(6), 0)
                ),
            )
        )

    # Calculate aggregated stats
    total_image_count = sum(v.image_count for v in versions)
    total_rated_count = sum(v.rated_count for v in versions)
    total_high_rated_count = sum(v.high_rated_count for v in versions)

    # Weighted average rating
    total_rating_sum = sum(
        (v.avg_rating or 0) * v.rated_count for v in versions if v.avg_rating
    )
    avg_rating = (
        round(total_rating_sum / total_rated_count, 2) if total_rated_count > 0 else None
    )

    # Aggregate rating distribution
    rating_distribution = dict.fromkeys(range(6), 0)
    for dist in version_rating_dist.values():
        for rating, count in dist.items():
            rating_distribution[rating] += count

    # Top samplers (aggregated across all versions)
    sampler_result = await db.execute(
        select(
            Image.sampler_name,
            func.count(Image.id).label("count"),
            func.avg(case((Image.rating > 0, Image.rating))).label("avg_rating"),
        )
        .where(base_filter)
        .where(versions_filter)
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

    # Top LoRAs (aggregated across all versions)
    lora_unnest = (
        select(
            Image.id,
            Image.rating,
            func.jsonb_array_elements(Image.loras).op("->>")("name").label("lora_name"),
        )
        .where(base_filter)
        .where(versions_filter)
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
        display_name=model_name,
        model_type=model_type,
        image_count=total_image_count,
        rated_count=total_rated_count,
        avg_rating=avg_rating,
        high_rated_count=total_high_rated_count,
        rating_distribution=rating_distribution,
        top_samplers=top_samplers,
        top_loras=top_loras,
        versions=versions,
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
