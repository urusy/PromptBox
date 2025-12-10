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
    total_unrated: int
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


class RatingAnalysisItem(BaseModel):
    name: str
    avg_rating: float
    count: int
    high_rated_count: int  # Count of images with rating >= 4


class RatingAnalysisResponse(BaseModel):
    by_model: list[RatingAnalysisItem]
    by_sampler: list[RatingAnalysisItem]
    by_lora: list[RatingAnalysisItem]
    by_steps: list[RatingAnalysisItem]
    by_cfg: list[RatingAnalysisItem]
    filtered_by_model: str | None = None  # Model name if filtered


class ModelListResponse(BaseModel):
    models: list[str]


class ModelRatingDistributionItem(BaseModel):
    model_name: str
    rating_0: int
    rating_1: int
    rating_2: int
    rating_3: int
    rating_4: int
    rating_5: int
    total: int
    avg_rating: float | None


class ModelRatingDistributionResponse(BaseModel):
    items: list[ModelRatingDistributionItem]


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
            func.count(case((Image.rating == 0, 1))).label("unrated"),
            func.avg(case((Image.rating > 0, Image.rating))).label("avg_rating"),
        ).where(base_filter)
    )
    overview_row = overview_result.one()
    overview = StatsOverview(
        total_images=overview_row.total or 0,
        total_favorites=overview_row.favorites or 0,
        total_rated=overview_row.rated or 0,
        total_unrated=overview_row.unrated or 0,
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


@router.get("/models-for-analysis", response_model=ModelListResponse)
async def get_models_for_analysis(
    db: DbSession,
    _: CurrentUser,
    min_count: int = 5,
) -> ModelListResponse:
    """Get list of models with enough rated images for analysis.

    Returns models ordered by average rating (highest first) to match Best Models chart.
    """
    base_filter = Image.deleted_at.is_(None)
    rated_filter = Image.rating > 0

    result = await db.execute(
        select(Image.model_name)
        .where(base_filter)
        .where(rated_filter)
        .where(Image.model_name.isnot(None))
        .group_by(Image.model_name)
        .having(func.count(Image.id) >= min_count)
        .order_by(func.avg(Image.rating).desc())
    )
    models = [row[0] for row in result.all()]
    return ModelListResponse(models=models)


@router.get("/rating-analysis", response_model=RatingAnalysisResponse)
async def get_rating_analysis(
    db: DbSession,
    _: CurrentUser,
    min_count: int = 5,
    model_name: str | None = None,
) -> RatingAnalysisResponse:
    """Get rating analysis - which settings tend to get higher ratings.

    If model_name is provided, analyze only images using that model.
    """

    # Base filter: non-deleted, rated images only
    base_filter = Image.deleted_at.is_(None)
    rated_filter = Image.rating > 0

    # Optional model filter
    model_filter = Image.model_name == model_name if model_name else True

    # By model name (top 10 by avg rating, with minimum count)
    # Skip if filtering by specific model
    by_model: list[RatingAnalysisItem] = []
    if not model_name:
        model_result = await db.execute(
            select(
                Image.model_name,
                func.avg(Image.rating).label("avg_rating"),
                func.count(Image.id).label("count"),
                func.count(case((Image.rating >= 4, 1))).label("high_rated"),
            )
            .where(base_filter)
            .where(rated_filter)
            .where(Image.model_name.isnot(None))
            .group_by(Image.model_name)
            .having(func.count(Image.id) >= min_count)
            .order_by(func.avg(Image.rating).desc())
            .limit(10)
        )
        by_model = [
            RatingAnalysisItem(
                name=row.model_name,
                avg_rating=round(float(row.avg_rating), 2),
                count=row.count,
                high_rated_count=row.high_rated,
            )
            for row in model_result.all()
        ]

    # By sampler (top 10 by avg rating)
    sampler_result = await db.execute(
        select(
            Image.sampler_name,
            func.avg(Image.rating).label("avg_rating"),
            func.count(Image.id).label("count"),
            func.count(case((Image.rating >= 4, 1))).label("high_rated"),
        )
        .where(base_filter)
        .where(rated_filter)
        .where(model_filter)
        .where(Image.sampler_name.isnot(None))
        .group_by(Image.sampler_name)
        .having(func.count(Image.id) >= min_count)
        .order_by(func.avg(Image.rating).desc())
        .limit(10)
    )
    by_sampler = [
        RatingAnalysisItem(
            name=row.sampler_name,
            avg_rating=round(float(row.avg_rating), 2),
            count=row.count,
            high_rated_count=row.high_rated,
        )
        for row in sampler_result.all()
    ]

    # By LoRA (unnest and aggregate)
    lora_unnest = (
        select(
            Image.id,
            Image.rating,
            func.jsonb_array_elements(Image.loras).op("->>")(
                "name"
            ).label("lora_name"),
        )
        .where(base_filter)
        .where(rated_filter)
        .where(model_filter)
        .where(func.jsonb_array_length(Image.loras) > 0)
        .subquery()
    )
    lora_result = await db.execute(
        select(
            lora_unnest.c.lora_name,
            func.avg(lora_unnest.c.rating).label("avg_rating"),
            func.count().label("count"),
            func.count(case((lora_unnest.c.rating >= 4, 1))).label("high_rated"),
        )
        .group_by(lora_unnest.c.lora_name)
        .having(func.count() >= min_count)
        .order_by(func.avg(lora_unnest.c.rating).desc())
        .limit(10)
    )
    by_lora = [
        RatingAnalysisItem(
            name=row.lora_name,
            avg_rating=round(float(row.avg_rating), 2),
            count=row.count,
            high_rated_count=row.high_rated,
        )
        for row in lora_result.all()
    ]

    # By steps (group into ranges)
    steps_case = case(
        (Image.steps < 20, "< 20"),
        (Image.steps < 30, "20-29"),
        (Image.steps < 40, "30-39"),
        (Image.steps < 50, "40-49"),
        else_="50+",
    )
    steps_result = await db.execute(
        select(
            steps_case.label("steps_range"),
            func.avg(Image.rating).label("avg_rating"),
            func.count(Image.id).label("count"),
            func.count(case((Image.rating >= 4, 1))).label("high_rated"),
        )
        .where(base_filter)
        .where(rated_filter)
        .where(model_filter)
        .where(Image.steps.isnot(None))
        .group_by(steps_case)
        .having(func.count(Image.id) >= min_count)
        .order_by(func.avg(Image.rating).desc())
    )
    by_steps = [
        RatingAnalysisItem(
            name=row.steps_range,
            avg_rating=round(float(row.avg_rating), 2),
            count=row.count,
            high_rated_count=row.high_rated,
        )
        for row in steps_result.all()
    ]

    # By CFG scale (group into ranges)
    cfg_case = case(
        (Image.cfg_scale < 5, "< 5"),
        (Image.cfg_scale < 7, "5-6.9"),
        (Image.cfg_scale < 9, "7-8.9"),
        (Image.cfg_scale < 12, "9-11.9"),
        else_="12+",
    )
    cfg_result = await db.execute(
        select(
            cfg_case.label("cfg_range"),
            func.avg(Image.rating).label("avg_rating"),
            func.count(Image.id).label("count"),
            func.count(case((Image.rating >= 4, 1))).label("high_rated"),
        )
        .where(base_filter)
        .where(rated_filter)
        .where(model_filter)
        .where(Image.cfg_scale.isnot(None))
        .group_by(cfg_case)
        .having(func.count(Image.id) >= min_count)
        .order_by(func.avg(Image.rating).desc())
    )
    by_cfg = [
        RatingAnalysisItem(
            name=row.cfg_range,
            avg_rating=round(float(row.avg_rating), 2),
            count=row.count,
            high_rated_count=row.high_rated,
        )
        for row in cfg_result.all()
    ]

    return RatingAnalysisResponse(
        by_model=by_model,
        by_sampler=by_sampler,
        by_lora=by_lora,
        by_steps=by_steps,
        by_cfg=by_cfg,
        filtered_by_model=model_name,
    )


@router.get("/model-rating-distribution", response_model=ModelRatingDistributionResponse)
async def get_model_rating_distribution(
    db: DbSession,
    _: CurrentUser,
    min_count: int = 10,
    limit: int = 15,
) -> ModelRatingDistributionResponse:
    """Get rating distribution per model.

    Returns the count of images at each rating level (0-5) for each model.
    """
    base_filter = Image.deleted_at.is_(None)

    result = await db.execute(
        select(
            Image.model_name,
            func.count(case((Image.rating == 0, 1))).label("rating_0"),
            func.count(case((Image.rating == 1, 1))).label("rating_1"),
            func.count(case((Image.rating == 2, 1))).label("rating_2"),
            func.count(case((Image.rating == 3, 1))).label("rating_3"),
            func.count(case((Image.rating == 4, 1))).label("rating_4"),
            func.count(case((Image.rating == 5, 1))).label("rating_5"),
            func.count(Image.id).label("total"),
            func.avg(case((Image.rating > 0, Image.rating))).label("avg_rating"),
        )
        .where(base_filter)
        .where(Image.model_name.isnot(None))
        .group_by(Image.model_name)
        .having(func.count(Image.id) >= min_count)
        .order_by(func.count(Image.id).desc())
        .limit(limit)
    )

    items = [
        ModelRatingDistributionItem(
            model_name=row.model_name,
            rating_0=row.rating_0,
            rating_1=row.rating_1,
            rating_2=row.rating_2,
            rating_3=row.rating_3,
            rating_4=row.rating_4,
            rating_5=row.rating_5,
            total=row.total,
            avg_rating=round(float(row.avg_rating), 2) if row.avg_rating else None,
        )
        for row in result.all()
    ]

    return ModelRatingDistributionResponse(items=items)
