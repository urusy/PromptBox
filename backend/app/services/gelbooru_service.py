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
            logger.warning(f"Gelbooru API error for '{query}': {e}")
            return []
        except Exception as e:
            logger.error(f"Error fetching Gelbooru tags: {e}")
            return []


# Global singleton instance
gelbooru_service = GelbooruService()
