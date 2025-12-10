from datetime import datetime
from uuid import UUID

from pydantic import BaseModel, Field


class SearchFilters(BaseModel):
    """Search filter parameters that can be saved in a preset."""

    q: str | None = None
    source_tool: str | None = None
    model_type: str | None = None
    model_name: str | None = None
    sampler_name: str | None = None
    min_rating: int | None = Field(None, ge=0, le=5)
    exact_rating: int | None = Field(None, ge=0, le=5)
    is_favorite: bool | None = None
    needs_improvement: bool | None = None
    tags: list[str] | None = None
    lora_name: str | None = None
    is_xyz_grid: bool | None = None
    is_upscaled: bool | None = None
    min_width: int | None = Field(None, ge=1)
    min_height: int | None = Field(None, ge=1)
    sort_by: str | None = None
    sort_order: str | None = None


class SearchPresetCreate(BaseModel):
    """Schema for creating a new search preset."""

    name: str = Field(..., min_length=1, max_length=100)
    filters: SearchFilters


class SearchPresetUpdate(BaseModel):
    """Schema for updating an existing search preset."""

    name: str | None = Field(None, min_length=1, max_length=100)
    filters: SearchFilters | None = None


class SearchPresetResponse(BaseModel):
    """Schema for search preset response."""

    id: UUID
    name: str
    filters: SearchFilters
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True
