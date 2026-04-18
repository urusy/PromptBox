"""Gelbooru tag search proxy endpoint."""

from fastapi import APIRouter, Query

from app.api.deps import CurrentUser
from app.schemas.gelbooru import GelbooruTagSearchResponse
from app.services.gelbooru_service import gelbooru_service

router = APIRouter(prefix="/gelbooru", tags=["gelbooru"])


@router.get("/tags", response_model=GelbooruTagSearchResponse)
async def search_gelbooru_tags(
    _: CurrentUser,
    q: str = Query(..., min_length=2, description="Search query for tag name"),
    limit: int = Query(30, ge=1, le=100, description="Max number of results"),
) -> GelbooruTagSearchResponse:
    """Search Gelbooru tags by partial match.

    Proxies requests to Gelbooru API with caching.
    """
    tags = await gelbooru_service.search_tags(q, limit)
    return GelbooruTagSearchResponse(tags=tags, query=q)
