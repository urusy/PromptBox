"""CivitAI API service with in-memory caching."""

from typing import Literal

import httpx
from cachetools import TTLCache
from loguru import logger

from app.schemas.model import (
    CivitaiModelInfo,
    CivitaiImage,
    CivitaiRecommendedSettings,
)

# CivitAI API base URL
CIVITAI_API_BASE = "https://civitai.com/api/v1"

# In-memory cache (TTL: 24 hours)
# Key: "model:{name}" or "hash:{hash}"
_civitai_cache: TTLCache[str, CivitaiModelInfo | None] = TTLCache(maxsize=1000, ttl=86400)


class CivitaiService:
    """Service for interacting with CivitAI API."""

    def __init__(self) -> None:
        self._client: httpx.AsyncClient | None = None

    async def _get_client(self) -> httpx.AsyncClient:
        """Get or create HTTP client."""
        if self._client is None:
            self._client = httpx.AsyncClient(
                timeout=30.0,
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

    def _parse_model_response(
        self,
        data: dict,
        is_exact_match: bool = True,
    ) -> CivitaiModelInfo:
        """Parse CivitAI API response into our schema."""
        # Get first model version for details
        versions = data.get("modelVersions", [])
        first_version = versions[0] if versions else {}

        # Extract images
        images: list[CivitaiImage] = []
        for img in first_version.get("images", [])[:5]:  # Limit to 5 images
            images.append(
                CivitaiImage(
                    url=img.get("url", ""),
                    width=img.get("width"),
                    height=img.get("height"),
                    nsfw=img.get("nsfw", False) or img.get("nsfwLevel", 0) > 1,
                )
            )

        # Extract recommended settings
        trained_words = first_version.get("trainedWords", [])

        # Try to get recommended settings from metadata or files
        recommended_settings = None
        files = first_version.get("files", [])
        if files:
            first_file = files[0]
            metadata = first_file.get("metadata", {})
            if metadata:
                recommended_settings = CivitaiRecommendedSettings(
                    clip_skip=metadata.get("clipSkip"),
                    steps=metadata.get("steps"),
                    cfg_scale=metadata.get("cfgScale"),
                    sampler=metadata.get("sampler"),
                    vae=metadata.get("vae"),
                    strength=metadata.get("strength"),
                )

        # Get base model
        base_model = first_version.get("baseModel")

        # Build civitai URL
        civitai_url = f"https://civitai.com/models/{data.get('id')}"

        # Get download URL
        download_url = None
        if files:
            download_url = files[0].get("downloadUrl")

        return CivitaiModelInfo(
            civitai_id=data.get("id", 0),
            name=data.get("name", ""),
            description=data.get("description"),
            type=data.get("type", ""),
            nsfw=data.get("nsfw", False),
            creator=data.get("creator", {}).get("username"),
            base_model=base_model,
            images=images,
            recommended_settings=recommended_settings,
            trigger_words=trained_words,
            download_url=download_url,
            civitai_url=civitai_url,
            is_exact_match=is_exact_match,
        )

    def _parse_version_response(
        self,
        data: dict,
        is_exact_match: bool = True,
    ) -> CivitaiModelInfo:
        """Parse CivitAI model-versions API response."""
        # Extract images
        images: list[CivitaiImage] = []
        for img in data.get("images", [])[:5]:
            images.append(
                CivitaiImage(
                    url=img.get("url", ""),
                    width=img.get("width"),
                    height=img.get("height"),
                    nsfw=img.get("nsfw", False) or img.get("nsfwLevel", 0) > 1,
                )
            )

        trained_words = data.get("trainedWords", [])

        # Try to get recommended settings from files metadata
        recommended_settings = None
        files = data.get("files", [])
        if files:
            first_file = files[0]
            metadata = first_file.get("metadata", {})
            if metadata:
                recommended_settings = CivitaiRecommendedSettings(
                    clip_skip=metadata.get("clipSkip"),
                    steps=metadata.get("steps"),
                    cfg_scale=metadata.get("cfgScale"),
                    sampler=metadata.get("sampler"),
                    vae=metadata.get("vae"),
                    strength=metadata.get("strength"),
                )

        # Get model info from nested model object
        model_info = data.get("model", {})
        model_id = model_info.get("id") or data.get("modelId", 0)

        civitai_url = f"https://civitai.com/models/{model_id}"

        download_url = None
        if files:
            download_url = files[0].get("downloadUrl")

        return CivitaiModelInfo(
            civitai_id=model_id,
            name=model_info.get("name") or data.get("name", ""),
            description=data.get("description") or model_info.get("description"),
            type=model_info.get("type", ""),
            nsfw=model_info.get("nsfw", False),
            creator=None,  # Not included in version response
            base_model=data.get("baseModel"),
            images=images,
            recommended_settings=recommended_settings,
            trigger_words=trained_words,
            download_url=download_url,
            civitai_url=civitai_url,
            is_exact_match=is_exact_match,
        )

    async def get_model_by_hash(self, hash_value: str) -> CivitaiModelInfo | None:
        """Get model info by SHA256 hash.

        This is the most accurate way to find a model on CivitAI.
        """
        cache_key = f"hash:{hash_value}"
        if cache_key in _civitai_cache:
            logger.debug(f"Cache hit for hash: {hash_value}")
            return _civitai_cache[cache_key]

        logger.info(f"Fetching CivitAI info by hash: {hash_value}")

        try:
            client = await self._get_client()
            response = await client.get(
                f"{CIVITAI_API_BASE}/model-versions/by-hash/{hash_value}"
            )

            if response.status_code == 404:
                logger.debug(f"Model not found by hash: {hash_value}")
                _civitai_cache[cache_key] = None
                return None

            response.raise_for_status()
            data = response.json()

            info = self._parse_version_response(data, is_exact_match=True)
            _civitai_cache[cache_key] = info

            # Also cache by model name for future lookups
            name_cache_key = f"model:{info.name.lower()}"
            _civitai_cache[name_cache_key] = info

            return info

        except httpx.HTTPStatusError as e:
            logger.warning(f"CivitAI API error for hash {hash_value}: {e}")
            return None
        except Exception as e:
            logger.error(f"Error fetching CivitAI info by hash: {e}")
            return None

    async def get_model_info(
        self,
        name: str,
        model_type: Literal["Checkpoint", "LORA", "TextualInversion"] = "Checkpoint",
    ) -> CivitaiModelInfo | None:
        """Get model info by name.

        First tries exact match, then fuzzy search.
        """
        cache_key = f"model:{name.lower()}:{model_type}"
        if cache_key in _civitai_cache:
            logger.debug(f"Cache hit for model: {name}")
            return _civitai_cache[cache_key]

        logger.info(f"Fetching CivitAI info for: {name} (type={model_type})")

        # Try exact name search first
        info = await self._search_models(name, model_type, exact=True)

        if info:
            _civitai_cache[cache_key] = info
            return info

        # Fall back to fuzzy search
        logger.debug(f"Exact match not found, trying fuzzy search for: {name}")
        info = await self._search_models(name, model_type, exact=False)

        if info:
            # Mark as non-exact match
            info.is_exact_match = False
            _civitai_cache[cache_key] = info
            return info

        # Cache negative result
        _civitai_cache[cache_key] = None
        return None

    async def _search_models(
        self,
        query: str,
        model_type: str,
        exact: bool = False,
    ) -> CivitaiModelInfo | None:
        """Search for models on CivitAI."""
        try:
            client = await self._get_client()

            params = {
                "query": query,
                "types": model_type,
                "limit": 5 if exact else 10,
                "nsfw": "true",  # Include NSFW models
            }

            response = await client.get(f"{CIVITAI_API_BASE}/models", params=params)
            response.raise_for_status()
            data = response.json()

            items = data.get("items", [])
            if not items:
                return None

            # For exact match, check if first result matches closely
            if exact:
                first_item = items[0]
                first_name = first_item.get("name", "").lower()
                query_lower = query.lower()

                # Check for exact or close match
                if first_name == query_lower or query_lower in first_name:
                    return self._parse_model_response(first_item, is_exact_match=True)
                return None

            # For fuzzy search, return best match
            # Score by similarity (simple substring match)
            query_lower = query.lower()
            best_match = None
            best_score = 0

            for item in items:
                item_name = item.get("name", "").lower()
                score = 0

                # Exact match
                if item_name == query_lower:
                    score = 100
                # Query is substring of name
                elif query_lower in item_name:
                    score = 80
                # Name is substring of query
                elif item_name in query_lower:
                    score = 60
                # Some words match
                else:
                    query_words = set(query_lower.split())
                    name_words = set(item_name.split())
                    common = query_words & name_words
                    if common:
                        score = len(common) * 20

                if score > best_score:
                    best_score = score
                    best_match = item

            if best_match and best_score >= 20:
                return self._parse_model_response(best_match, is_exact_match=False)

            return None

        except httpx.HTTPStatusError as e:
            logger.warning(f"CivitAI API error for search '{query}': {e}")
            return None
        except Exception as e:
            logger.error(f"Error searching CivitAI: {e}")
            return None

    async def search_models(
        self,
        query: str,
        model_type: Literal["Checkpoint", "LORA", "TextualInversion"] | None = None,
        limit: int = 10,
    ) -> list[CivitaiModelInfo]:
        """Search for multiple models on CivitAI."""
        try:
            client = await self._get_client()

            params = {
                "query": query,
                "limit": limit,
                "nsfw": "true",
            }
            if model_type:
                params["types"] = model_type

            response = await client.get(f"{CIVITAI_API_BASE}/models", params=params)
            response.raise_for_status()
            data = response.json()

            results = []
            for item in data.get("items", []):
                info = self._parse_model_response(item)
                results.append(info)

            return results

        except Exception as e:
            logger.error(f"Error searching CivitAI: {e}")
            return []


# Global singleton instance
civitai_service = CivitaiService()
