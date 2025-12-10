from datetime import datetime, timedelta

from fastapi import APIRouter
from pydantic import BaseModel
from sqlalchemy import func, select, case, extract
from sqlalchemy.dialects.postgresql import JSONB

from app.api.deps import CurrentUser, DbSession
from app.models.image import Image

router = APIRouter(prefix="/stats", tags=["stats"])


class CountItem(BaseModel):
    name: str
    count: int


class TimeSeriesItem(BaseModel):
    date: str
    count: int


class RatingDistribution(BaseModel):
    rating: int
    count: int


class StatsOverview(BaseModel):
    total_images: int
    total_favorites: int
    total_rated: int
    avg_rating: float | None


class StatsResponse(BaseModel):
    overview: StatsOverview
    by_model_type: list[CountItem]
    by_source_tool: list[CountItem]
    by_model_name: list[CountItem]
    by_sampler: list[CountItem]
    by_lora: list[CountItem]
    by_rating: list[RatingDistribution]
    daily_counts: list[TimeSeriesItem]


@router.get("", response_model=StatsResponse)
async def get_stats(
    db: DbSession,
    _: CurrentUser,
    days: int = 30,
) -> StatsResponse:
    """Get usage statistics for the image library."""

    # Base filter: non-deleted images
    base_filter = Image.deleted_at.is_(None)

    # Overview stats
    overview_result = await db.execute(
        select(
            func.count(Image.id).label("total"),
            func.count(case((Image.is_favorite == True, 1))).label("favorites"),  # noqa: E712
            func.count(case((Image.rating > 0, 1))).label("rated"),
            func.avg(case((Image.rating > 0, Image.rating))).label("avg_rating"),
        ).where(base_filter)
    )
    overview_row = overview_result.one()
    overview = StatsOverview(
        total_images=overview_row.total or 0,
        total_favorites=overview_row.favorites or 0,
        total_rated=overview_row.rated or 0,
        avg_rating=round(float(overview_row.avg_rating), 2) if overview_row.avg_rating else None,
    )

    # By model type
    model_type_result = await db.execute(
        select(Image.model_type, func.count(Image.id).label("count"))
        .where(base_filter)
        .where(Image.model_type.isnot(None))
        .group_by(Image.model_type)
        .order_by(func.count(Image.id).desc())
        .limit(10)
    )
    by_model_type = [
        CountItem(name=row.model_type, count=row.count)
        for row in model_type_result.all()
    ]

    # By source tool
    source_tool_result = await db.execute(
        select(Image.source_tool, func.count(Image.id).label("count"))
        .where(base_filter)
        .group_by(Image.source_tool)
        .order_by(func.count(Image.id).desc())
    )
    by_source_tool = [
        CountItem(name=row.source_tool, count=row.count)
        for row in source_tool_result.all()
    ]

    # By model name (top 10)
    model_name_result = await db.execute(
        select(Image.model_name, func.count(Image.id).label("count"))
        .where(base_filter)
        .where(Image.model_name.isnot(None))
        .group_by(Image.model_name)
        .order_by(func.count(Image.id).desc())
        .limit(10)
    )
    by_model_name = [
        CountItem(name=row.model_name, count=row.count)
        for row in model_name_result.all()
    ]

    # By sampler (top 10)
    sampler_result = await db.execute(
        select(Image.sampler_name, func.count(Image.id).label("count"))
        .where(base_filter)
        .where(Image.sampler_name.isnot(None))
        .group_by(Image.sampler_name)
        .order_by(func.count(Image.id).desc())
        .limit(10)
    )
    by_sampler = [
        CountItem(name=row.sampler_name, count=row.count)
        for row in sampler_result.all()
    ]

    # By LoRA (unnest JSONB array and count)
    # loras is JSONB array like [{"name": "lora1", "weight": 0.8}, ...]
    lora_unnest = (
        select(
            func.jsonb_array_elements(Image.loras).op("->>")(
                "name"
            ).label("lora_name")
        )
        .where(base_filter)
        .where(func.jsonb_array_length(Image.loras) > 0)
        .subquery()
    )
    lora_result = await db.execute(
        select(lora_unnest.c.lora_name, func.count().label("count"))
        .group_by(lora_unnest.c.lora_name)
        .order_by(func.count().desc())
        .limit(10)
    )
    by_lora = [
        CountItem(name=row.lora_name, count=row.count)
        for row in lora_result.all()
    ]

    # Rating distribution
    rating_result = await db.execute(
        select(Image.rating, func.count(Image.id).label("count"))
        .where(base_filter)
        .group_by(Image.rating)
        .order_by(Image.rating)
    )
    by_rating = [
        RatingDistribution(rating=row.rating, count=row.count)
        for row in rating_result.all()
    ]

    # Daily counts for last N days
    start_date = datetime.utcnow() - timedelta(days=days)
    day_trunc = func.date_trunc("day", Image.created_at)
    daily_result = await db.execute(
        select(
            day_trunc.label("day"),
            func.count(Image.id).label("count"),
        )
        .where(base_filter)
        .where(Image.created_at >= start_date)
        .group_by(day_trunc)
        .order_by(day_trunc)
    )
    daily_counts = [
        TimeSeriesItem(date=row.day.strftime("%Y-%m-%d"), count=row.count)
        for row in daily_result.all()
    ]

    return StatsResponse(
        overview=overview,
        by_model_type=by_model_type,
        by_source_tool=by_source_tool,
        by_model_name=by_model_name,
        by_sampler=by_sampler,
        by_lora=by_lora,
        by_rating=by_rating,
        daily_counts=daily_counts,
    )
