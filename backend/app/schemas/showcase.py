from datetime import datetime
from uuid import UUID

from pydantic import BaseModel, Field


class ShowcaseCreate(BaseModel):
    """Schema for creating a new showcase."""

    name: str = Field(..., min_length=1, max_length=100)
    description: str | None = Field(None, max_length=1000)
    icon: str | None = Field(None, max_length=50)


class ShowcaseUpdate(BaseModel):
    """Schema for updating an existing showcase."""

    name: str | None = Field(None, min_length=1, max_length=100)
    description: str | None = Field(None, max_length=1000)
    icon: str | None = Field(None, max_length=50)
    cover_image_id: UUID | None = None


class ShowcaseImageAdd(BaseModel):
    """Schema for adding images to a showcase."""

    image_ids: list[UUID] = Field(..., min_length=1, max_length=100)


class ShowcaseImageRemove(BaseModel):
    """Schema for removing images from a showcase."""

    image_ids: list[UUID] = Field(..., min_length=1, max_length=100)


class ShowcaseImageReorder(BaseModel):
    """Schema for reordering images in a showcase."""

    image_ids: list[UUID] = Field(..., min_length=1, description="Image IDs in new order")


class ShowcaseImageInfo(BaseModel):
    """Brief info about an image in a showcase."""

    id: UUID
    storage_path: str
    thumbnail_path: str
    sort_order: int
    added_at: datetime


class ShowcaseResponse(BaseModel):
    """Schema for showcase response."""

    id: UUID
    name: str
    description: str | None
    icon: str | None
    cover_image_id: UUID | None
    cover_thumbnail_path: str | None
    image_count: int
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True


class ShowcaseDetailResponse(ShowcaseResponse):
    """Schema for showcase detail response with images."""

    images: list[ShowcaseImageInfo]


class ShowcaseImageCheck(BaseModel):
    """Schema for checking which images are already in showcases."""

    image_ids: list[UUID] = Field(..., min_length=1, max_length=100)


class ShowcaseImageCheckResult(BaseModel):
    """Result of checking which images are in a showcase."""

    showcase_id: UUID
    existing_count: int
