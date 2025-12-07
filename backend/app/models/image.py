from datetime import datetime
from decimal import Decimal
from typing import Any
from uuid import UUID

from sqlalchemy import (
    Boolean,
    BigInteger,
    DateTime,
    Integer,
    Numeric,
    SmallInteger,
    String,
    Text,
)
from sqlalchemy.dialects.postgresql import JSONB, UUID as PG_UUID
from sqlalchemy.orm import Mapped, mapped_column

from app.database import Base


class Image(Base):
    __tablename__ = "images"

    # Primary key
    id: Mapped[UUID] = mapped_column(PG_UUID(as_uuid=True), primary_key=True)

    # Source info
    source_tool: Mapped[str] = mapped_column(String(20), nullable=False)
    model_type: Mapped[str | None] = mapped_column(String(20))
    has_metadata: Mapped[bool] = mapped_column(Boolean, default=True, nullable=False)

    # File info
    original_filename: Mapped[str] = mapped_column(String(255), nullable=False)
    storage_path: Mapped[str] = mapped_column(String(512), nullable=False)
    thumbnail_path: Mapped[str] = mapped_column(String(512), nullable=False)
    file_hash: Mapped[str] = mapped_column(String(64), nullable=False, unique=True)
    width: Mapped[int] = mapped_column(Integer, nullable=False)
    height: Mapped[int] = mapped_column(Integer, nullable=False)
    file_size_bytes: Mapped[int] = mapped_column(BigInteger, nullable=False)

    # Common search columns
    positive_prompt: Mapped[str | None] = mapped_column(Text)
    negative_prompt: Mapped[str | None] = mapped_column(Text)
    model_name: Mapped[str | None] = mapped_column(String(255))
    sampler_name: Mapped[str | None] = mapped_column(String(50))
    scheduler: Mapped[str | None] = mapped_column(String(50))
    steps: Mapped[int | None] = mapped_column(Integer)
    cfg_scale: Mapped[Decimal | None] = mapped_column(Numeric(5, 2))
    seed: Mapped[int | None] = mapped_column(BigInteger)

    # Extended data (JSONB)
    loras: Mapped[list[dict[str, Any]]] = mapped_column(JSONB, default=list, nullable=False)
    controlnets: Mapped[list[dict[str, Any]]] = mapped_column(JSONB, default=list, nullable=False)
    embeddings: Mapped[list[dict[str, Any]]] = mapped_column(JSONB, default=list, nullable=False)
    model_params: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)
    workflow_extras: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)

    # Raw metadata
    raw_metadata: Mapped[dict[str, Any] | None] = mapped_column(JSONB)

    # User data
    rating: Mapped[int] = mapped_column(SmallInteger, default=0, nullable=False)
    is_favorite: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    needs_improvement: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    user_tags: Mapped[list[str]] = mapped_column(JSONB, default=list, nullable=False)
    user_memo: Mapped[str | None] = mapped_column(Text)

    # Timestamps
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=datetime.utcnow, nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=datetime.utcnow, nullable=False
    )
    deleted_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
