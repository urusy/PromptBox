from fastapi import APIRouter, Query
from sqlalchemy import func, select, text

from app.api.deps import CurrentUser, DbSession
from app.models.image import Image

router = APIRouter(prefix="/tags", tags=["tags"])


@router.get("", response_model=list[str])
async def list_tags(
    db: DbSession,
    _: CurrentUser,
    limit: int = Query(10, ge=1, le=100),
) -> list[str]:
    """
    Get most recently used tags.
    Returns unique tags ordered by most recent usage (based on image updated_at).
    """
    # user_tags is JSONB, use jsonb_array_elements_text to unnest
    # Using a subquery to get the max updated_at for each tag
    unnested = (
        select(
            func.jsonb_array_elements_text(Image.user_tags).label("tag"),
            Image.updated_at,
        )
        .where(Image.deleted_at.is_(None))
        .where(func.jsonb_array_length(Image.user_tags) > 0)
        .subquery()
    )

    result = await db.execute(
        select(unnested.c.tag)
        .group_by(unnested.c.tag)
        .order_by(func.max(unnested.c.updated_at).desc())
        .limit(limit)
    )

    return list(result.scalars().all())
