"""Schemas for Model and LoRA resources."""

from pydantic import BaseModel


class ModelListItem(BaseModel):
    """Model item in list view."""

    name: str
    display_name: str  # Name without path (after last \ or /)
    model_type: str | None
    image_count: int
    rated_count: int
    avg_rating: float | None
    high_rated_count: int  # Rating >= 4


class ModelListResponse(BaseModel):
    """Response for model list."""

    items: list[ModelListItem]
    total: int


class ModelDetail(BaseModel):
    """Model detail with statistics."""

    name: str
    display_name: str
    model_type: str | None
    image_count: int
    rated_count: int
    avg_rating: float | None
    high_rated_count: int
    rating_distribution: dict[int, int]  # {0: count, 1: count, ...}
    top_samplers: list[dict]  # [{name, count, avg_rating}]
    top_loras: list[dict]  # [{name, count, avg_rating}]


class LoraListItem(BaseModel):
    """LoRA item in list view."""

    name: str
    display_name: str  # Name without path
    hash: str | None  # SHA256 hash if available
    image_count: int
    rated_count: int
    avg_rating: float | None
    high_rated_count: int


class LoraListResponse(BaseModel):
    """Response for LoRA list."""

    items: list[LoraListItem]
    total: int


class LoraDetail(BaseModel):
    """LoRA detail with statistics."""

    name: str
    display_name: str
    hash: str | None
    image_count: int
    rated_count: int
    avg_rating: float | None
    high_rated_count: int
    rating_distribution: dict[int, int]
    avg_weight: float | None
    top_models: list[dict]  # [{name, count, avg_rating}]
    top_samplers: list[dict]


class CivitaiImage(BaseModel):
    """CivitAI model image."""

    url: str
    width: int | None = None
    height: int | None = None
    nsfw: bool = False


class CivitaiRecommendedSettings(BaseModel):
    """Recommended generation settings from CivitAI."""

    clip_skip: int | None = None
    steps: int | None = None
    cfg_scale: float | None = None
    sampler: str | None = None
    vae: str | None = None
    strength: float | None = None  # For LoRA


class CivitaiVersionInfo(BaseModel):
    """CivitAI model version information."""

    version_id: int
    name: str
    base_model: str | None = None  # SD 1.5, SDXL, Illustrious, etc.
    images: list[CivitaiImage] = []
    recommended_settings: CivitaiRecommendedSettings | None = None
    trigger_words: list[str] = []  # For LoRA
    download_url: str | None = None
    file_size_kb: float | None = None
    published_at: str | None = None


class CivitaiModelInfo(BaseModel):
    """CivitAI model information."""

    civitai_id: int
    name: str
    description: str | None = None
    type: str  # Checkpoint, LORA, etc.
    nsfw: bool = False
    creator: str | None = None
    civitai_url: str | None = None
    is_exact_match: bool = True  # False if found via fuzzy search
    versions: list[CivitaiVersionInfo] = []  # All versions of the model


class CivitaiInfoResponse(BaseModel):
    """Response for CivitAI info endpoint."""

    found: bool
    info: CivitaiModelInfo | None = None
    error: str | None = None
