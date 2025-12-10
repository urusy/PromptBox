from datetime import datetime
from typing import Any
from uuid import UUID

from sqlalchemy import DateTime, String
from sqlalchemy.dialects.postgresql import JSONB, UUID as PG_UUID
from sqlalchemy.orm import Mapped, mapped_column

from app.database import Base


class SmartFolder(Base):
    __tablename__ = "smart_folders"

    # Primary key
    id: Mapped[UUID] = mapped_column(PG_UUID(as_uuid=True), primary_key=True)

    # Name
    name: Mapped[str] = mapped_column(String(100), nullable=False)

    # Icon (emoji or icon name)
    icon: Mapped[str | None] = mapped_column(String(50), nullable=True)

    # Filters (JSONB) - same structure as SearchFilters
    filters: Mapped[dict[str, Any]] = mapped_column(JSONB, default=dict, nullable=False)

    # Timestamps
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=datetime.utcnow, nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=datetime.utcnow, nullable=False
    )
