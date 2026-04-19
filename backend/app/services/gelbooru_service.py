"""Gelbooru Tag API service with in-memory caching."""

import httpx
from cachetools import TTLCache
from loguru import logger

from app.config import get_settings
from app.schemas.gelbooru import GelbooruTag

# Gelbooru API base URL
GELBOORU_API_BASE = "https://gelbooru.com/index.php"

# In-memory cache (TTL: 5 minutes)
_gelbooru_cache: TTLCache[str, list[GelbooruTag]] = TTLCache(maxsize=500, ttl=300)


class GelbooruServiceError(Exception):
    """Base exception for Gelbooru service."""


class GelbooruRateLimitError(GelbooruServiceError):
    """Raised when Gelbooru API returns a rate-limit response (HTTP 429)."""


class GelbooruUpstreamError(GelbooruServiceError):
    """Raised when Gelbooru API returns a non-2xx, non-429 HTTP response."""


class GelbooruUnavailableError(GelbooruServiceError):
    """Raised on network-level failures (timeout, connection error, etc.)."""


class GelbooruService:
    """Service for interacting with Gelbooru Tag API."""

    def __init__(self) -> None:
        self._client: httpx.AsyncClient | None = None

    async def _get_client(self) -> httpx.AsyncClient:
        """Get or create HTTP client."""
        if self._client is None:
            self._client = httpx.AsyncClient(
                timeout=10.0,
                headers={
                    "User-Agent": "PromptBox/1.0",
                },
            )
        return self._client

    async def close(self) -> None:
        """Close HTTP client."""
        if self._client is not None:
            await self._client.aclose()
            self._client = None

    async def search_tags(self, query: str, limit: int = 30) -> list[GelbooruTag]:
        """Search Gelbooru tags by partial match.

        Uses name_pattern with % wildcards for partial matching.
        Results are ordered by count (descending).
        """
        cache_key = f"tags:{query}:{limit}"
        if cache_key in _gelbooru_cache:
            logger.debug(f"Cache hit for Gelbooru tags: {query}")
            return _gelbooru_cache[cache_key]

        logger.info(f"Fetching Gelbooru tags for: {query}")

        try:
            client = await self._get_client()
            settings = get_settings()
            params = {
                "page": "dapi",
                "s": "tag",
                "q": "index",
                "name_pattern": f"%{query}%",
                "json": "1",
                "orderby": "count",
                "order": "DESC",
                "limit": limit,
                "api_key": settings.gelbooru_api_key,
                "user_id": settings.gelbooru_user_id,
            }

            response = await client.get(GELBOORU_API_BASE, params=params)
            response.raise_for_status()

            data = response.json()

            # Handle response format variations
            # Can be a list directly, or {"tag": [...]}
            raw_tags: list[dict] = []
            if isinstance(data, list):
                raw_tags = data
            elif isinstance(data, dict):
                tag_data = data.get("tag", [])
                if isinstance(tag_data, list):
                    raw_tags = tag_data
                elif isinstance(tag_data, dict):
                    raw_tags = [tag_data]
            # Empty response (e.g. no results) returns empty string or empty object
            # raw_tags stays empty in that case

            tags = [
                GelbooruTag(
                    id=t.get("id", 0),
                    name=t.get("name", ""),
                    count=t.get("count", 0),
                    type=t.get("type", 0),
                    ambiguous=bool(t.get("ambiguous", False)),
                )
                for t in raw_tags
            ]

            _gelbooru_cache[cache_key] = tags
            return tags

        except httpx.HTTPStatusError as e:
            status_code = e.response.status_code
            if status_code == 429:
                logger.warning(f"Gelbooru rate limit hit for '{query}'")
                raise GelbooruRateLimitError("Gelbooru API rate limit exceeded") from e
            logger.warning(f"Gelbooru API HTTP {status_code} for '{query}': {e}")
            raise GelbooruUpstreamError(
                f"Gelbooru API returned HTTP {status_code}"
            ) from e
        except httpx.TimeoutException as e:
            logger.warning(f"Gelbooru API timeout for '{query}': {e}")
            raise GelbooruUnavailableError("Gelbooru API timeout") from e
        except httpx.HTTPError as e:
            logger.warning(f"Gelbooru API network error for '{query}': {e}")
            raise GelbooruUnavailableError("Gelbooru API network error") from e
        except (ValueError, KeyError) as e:
            # JSON decode or response format errors
            logger.error(f"Gelbooru API response parse error for '{query}': {e}")
            raise GelbooruUpstreamError("Gelbooru API returned unexpected data") from e


# Global singleton instance
gelbooru_service = GelbooruService()
