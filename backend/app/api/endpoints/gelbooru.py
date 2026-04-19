"""Gelbooru tag search proxy endpoint."""

from fastapi import APIRouter, HTTPException, Query, status

from app.api.deps import CurrentUser
from app.schemas.gelbooru import GelbooruTagSearchResponse
from app.services.gelbooru_service import (
    GelbooruRateLimitError,
    GelbooruUnavailableError,
    GelbooruUpstreamError,
    gelbooru_service,
)

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
    try:
        tags = await gelbooru_service.search_tags(q, limit)
    except GelbooruRateLimitError as e:
        raise HTTPException(
            status_code=status.HTTP_429_TOO_MANY_REQUESTS,
            detail="Gelbooru API rate limit exceeded. Please try again later.",
        ) from e
    except GelbooruUnavailableError as e:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="Gelbooru API is temporarily unavailable.",
        ) from e
    except GelbooruUpstreamError as e:
        raise HTTPException(
            status_code=status.HTTP_502_BAD_GATEWAY,
            detail="Gelbooru API returned an unexpected response.",
        ) from e
    return GelbooruTagSearchResponse(tags=tags, query=q)
