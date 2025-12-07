from datetime import datetime
from decimal import Decimal
from typing import Any
from uuid import UUID

from pydantic import BaseModel, Field


class LoraInfo(BaseModel):
    name: str
    weight: float
    weight_clip: float | None = None
    hash: str | None = None


class ControlNetInfo(BaseModel):
    model: str
    weight: float
    guidance_start: float = 0.0
    guidance_end: float = 1.0
    preprocessor: str | None = None


class EmbeddingInfo(BaseModel):
    name: str
    hash: str | None = None


class ImageBase(BaseModel):
    source_tool: str
    model_type: str | None = None
    has_metadata: bool = True
    original_filename: str
    file_hash: str
    width: int
    height: int
    file_size_bytes: int
    positive_prompt: str | None = None
    negative_prompt: str | None = None
    model_name: str | None = None
    sampler_name: str | None = None
    scheduler: str | None = None
    steps: int | None = None
    cfg_scale: Decimal | None = None
    seed: int | None = None
    loras: list[LoraInfo] = Field(default_factory=list)
    controlnets: list[ControlNetInfo] = Field(default_factory=list)
    embeddings: list[EmbeddingInfo] = Field(default_factory=list)
    model_params: dict[str, Any] = Field(default_factory=dict)
    workflow_extras: dict[str, Any] = Field(default_factory=dict)
    raw_metadata: dict[str, Any] | None = None


class ImageCreate(ImageBase):
    storage_path: str
    thumbnail_path: str


class ImageUpdate(BaseModel):
    rating: int | None = Field(None, ge=0, le=5)
    is_favorite: bool | None = None
    needs_improvement: bool | None = None
    user_tags: list[str] | None = None
    user_memo: str | None = None


class ImageResponse(BaseModel):
    id: UUID
    source_tool: str
    model_type: str | None
    has_metadata: bool
    original_filename: str
    storage_path: str
    thumbnail_path: str
    file_hash: str
    width: int
    height: int
    file_size_bytes: int
    positive_prompt: str | None
    negative_prompt: str | None
    model_name: str | None
    sampler_name: str | None
    scheduler: str | None
    steps: int | None
    cfg_scale: Decimal | None
    seed: int | None
    loras: list[dict[str, Any]]
    controlnets: list[dict[str, Any]]
    embeddings: list[dict[str, Any]]
    model_params: dict[str, Any]
    workflow_extras: dict[str, Any]
    raw_metadata: dict[str, Any] | None
    rating: int
    is_favorite: bool
    needs_improvement: bool
    user_tags: list[str]
    user_memo: str | None
    created_at: datetime
    updated_at: datetime
    deleted_at: datetime | None

    class Config:
        from_attributes = True


class ImageListResponse(BaseModel):
    id: UUID
    source_tool: str
    model_type: str | None
    thumbnail_path: str
    width: int
    height: int
    model_name: str | None
    rating: int
    is_favorite: bool
    created_at: datetime

    class Config:
        from_attributes = True


class ImageSearchParams(BaseModel):
    q: str | None = None  # Full-text search in prompts
    source_tool: str | None = None
    model_type: str | None = None
    model_name: str | None = None
    sampler_name: str | None = None
    min_rating: int | None = Field(None, ge=0, le=5)
    is_favorite: bool | None = None
    needs_improvement: bool | None = None
    tags: list[str] | None = None
    lora_name: str | None = None
    include_deleted: bool = False
    page: int = Field(1, ge=1)
    per_page: int = Field(24, ge=1, le=100)
    sort_by: str = "created_at"
    sort_order: str = "desc"
