import re

from cachetools import TTLCache
from fastapi import APIRouter, Query
from sqlalchemy import func, select

from app.api.deps import CurrentUser, DbSession
from app.models.image import Image


def escape_like_pattern(value: str) -> str:
    """Escape special characters in LIKE patterns (%, _, \\)."""
    return re.sub(r"([%_\\])", r"\\\1", value)


router = APIRouter(prefix="/tags", tags=["tags"])

# In-memory cache for tag lists (TTL: 5 minutes)
_tag_cache: TTLCache[str, list[str]] = TTLCache(maxsize=100, ttl=300)


def invalidate_tag_cache() -> None:
    """Clear the tag cache. Call this when tags are modified."""
    _tag_cache.clear()


@router.get("", response_model=list[str])
async def list_tags(
    db: DbSession,
    _: CurrentUser,
    q: str | None = Query(None, description="Search query to filter tags"),
    limit: int = Query(10, ge=1, le=100),
) -> list[str]:
    """
    Get tags with optional search filtering.

    - Without query: Returns most recently used tags (up to limit)
    - With query: Returns all tags matching the query (up to limit)
    """
    # Check cache first
    cache_key = f"{q or 'all'}:{limit}"
    if cache_key in _tag_cache:
        return _tag_cache[cache_key]

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

    query = select(unnested.c.tag).group_by(unnested.c.tag)

    # If search query provided, filter tags
    if q:
        escaped_q = escape_like_pattern(q)
        query = query.where(unnested.c.tag.ilike(f"%{escaped_q}%", escape="\\"))

    # Order by most recent usage
    query = query.order_by(func.max(unnested.c.updated_at).desc()).limit(limit)

    result = await db.execute(query)
    tags = list(result.scalars().all())

    # Store in cache
    _tag_cache[cache_key] = tags

    return tags
