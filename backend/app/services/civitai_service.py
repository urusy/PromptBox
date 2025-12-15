"""CivitAI API service with in-memory caching."""

import re
from typing import Literal

import httpx
from cachetools import TTLCache
from loguru import logger

from app.schemas.model import (
    CivitaiImage,
    CivitaiModelInfo,
    CivitaiRecommendedSettings,
)

# CivitAI API base URL
CIVITAI_API_BASE = "https://civitai.com/api/v1"


def normalize_model_name(name: str) -> str:
    """Normalize model name for better CivitAI search matching.

    - Remove version suffixes like _v50, _v20VAE, v6, etc.
    - Split camelCase into separate words
    - Remove underscores and hyphens
    - Convert to lowercase
    """
    # Remove common version patterns
    # Patterns: _v50, _v20VAE, v6, V2.0, _v1.5, etc.
    cleaned = re.sub(r"[_-]?v\d+(\.\d+)?[a-zA-Z]*$", "", name, flags=re.IGNORECASE)

    # Split camelCase: "cyberIllustrious" -> "cyber Illustrious"
    cleaned = re.sub(r"([a-z])([A-Z])", r"\1 \2", cleaned)

    # Replace underscores and hyphens with spaces
    cleaned = re.sub(r"[_-]+", " ", cleaned)

    # Remove extra whitespace and convert to lowercase
    cleaned = " ".join(cleaned.split()).lower()

    return cleaned


def extract_words(text: str) -> set[str]:
    """Extract words from text, handling various separators and camelCase."""
    # First, split camelCase: "cyberIllustrious" -> "cyber Illustrious"
    text = re.sub(r"([a-z])([A-Z])", r"\1 \2", text)
    # Split on spaces, underscores, hyphens, and numbers adjacent to letters
    text = re.sub(r"(\D)(\d)", r"\1 \2", text)  # letter followed by number
    text = re.sub(r"(\d)(\D)", r"\1 \2", text)  # number followed by letter
    words = re.split(r"[\s_-]+", text.lower())
    # Filter out very short words and version-like strings
    return {w for w in words if len(w) > 2 and not re.match(r"^v?\d+", w)}

# In-memory cache (TTL: 24 hours)
# Key: "model:{name}" or "hash:{hash}"
_civitai_cache: TTLCache[str, CivitaiModelInfo | None] = TTLCache(
    maxsize=1000, ttl=86400
)


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

            # Normalize query for better search results
            normalized_query = normalize_model_name(query)
            logger.debug(f"Searching CivitAI: original='{query}', normalized='{normalized_query}'")

            params = {
                "query": normalized_query,
                "types": model_type,
                "limit": 10,  # Get more results for better matching
                "nsfw": "true",  # Include NSFW models
            }

            response = await client.get(f"{CIVITAI_API_BASE}/models", params=params)
            response.raise_for_status()
            data = response.json()

            items = data.get("items", [])
            if not items:
                logger.debug(f"No results from CivitAI for: {normalized_query}")
                return None

            # Extract words from query for matching
            query_words = extract_words(query)
            query_normalized = normalize_model_name(query)

            best_match = None
            best_score = 0

            for item in items:
                item_name = item.get("name", "")
                item_normalized = normalize_model_name(item_name)
                item_words = extract_words(item_name)

                score = 0

                # Exact normalized match
                if item_normalized == query_normalized:
                    score = 100
                # Normalized query is substring of normalized name
                elif query_normalized in item_normalized:
                    score = 90
                # Normalized name is substring of normalized query
                elif item_normalized in query_normalized:
                    score = 85
                else:
                    # Word-based matching (improved)
                    common = query_words & item_words
                    if common:
                        # Score based on percentage of query words matched
                        match_ratio = len(common) / max(len(query_words), 1)
                        # Also consider how many of the item's significant words match
                        item_match_ratio = len(common) / max(len(item_words), 1)
                        # Combined score
                        score = int((match_ratio * 50) + (item_match_ratio * 30))

                        # Bonus for matching significant/long words
                        for word in common:
                            if len(word) >= 5:
                                score += 10
                    else:
                        # Fallback: check if query words are substrings of concatenated item name
                        # This helps with compound words like "cyberillustrious" matching
                        # "CyberRealistic CyberIllustrious"
                        item_concat = item_normalized.replace(" ", "")
                        for qword in query_words:
                            if len(qword) >= 5 and qword in item_concat:
                                score = max(score, 60)  # Significant substring match
                                break

                        # Also check reverse: if item words are in query
                        query_concat = query_normalized.replace(" ", "")
                        for iword in item_words:
                            if len(iword) >= 5 and iword in query_concat:
                                score = max(score, 55)
                                break

                if score > best_score:
                    best_score = score
                    best_match = item
                    logger.debug(f"  Candidate: '{item_name}' score={score}")

            # For exact mode, require higher score
            min_score = 80 if exact else 30

            if best_match and best_score >= min_score:
                is_exact = best_score >= 85
                logger.info(
                    f"CivitAI match for '{query}': '{best_match.get('name')}' "
                    f"(score={best_score}, exact={is_exact})"
                )
                return self._parse_model_response(best_match, is_exact_match=is_exact)

            logger.debug(f"No good match found for '{query}' (best_score={best_score})")
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
