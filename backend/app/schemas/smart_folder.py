from datetime import datetime
from uuid import UUID

from pydantic import BaseModel, Field

from app.schemas.search_preset import SearchFilters


class SmartFolderCreate(BaseModel):
    """Schema for creating a new smart folder."""

    name: str = Field(..., min_length=1, max_length=100)
    icon: str | None = Field(None, max_length=50)
    filters: SearchFilters


class SmartFolderUpdate(BaseModel):
    """Schema for updating an existing smart folder."""

    name: str | None = Field(None, min_length=1, max_length=100)
    icon: str | None = Field(None, max_length=50)
    filters: SearchFilters | None = None


class SmartFolderResponse(BaseModel):
    """Schema for smart folder response."""

    id: UUID
    name: str
    icon: str | None
    filters: SearchFilters
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True
