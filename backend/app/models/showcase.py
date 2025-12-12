from datetime import datetime
from uuid import UUID

from sqlalchemy import DateTime, ForeignKey, Integer, String, Text
from sqlalchemy.dialects.postgresql import UUID as PG_UUID
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.database import Base


class Showcase(Base):
    """Showcase model - a collection of curated images."""

    __tablename__ = "showcases"

    # Primary key
    id: Mapped[UUID] = mapped_column(PG_UUID(as_uuid=True), primary_key=True)

    # Name and description
    name: Mapped[str] = mapped_column(String(100), nullable=False)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)

    # Icon (emoji or icon name)
    icon: Mapped[str | None] = mapped_column(String(50), nullable=True)

    # Cover image ID (optional)
    cover_image_id: Mapped[UUID | None] = mapped_column(
        PG_UUID(as_uuid=True),
        ForeignKey("images.id", ondelete="SET NULL"),
        nullable=True,
    )

    # Timestamps
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=datetime.utcnow, nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=datetime.utcnow, nullable=False
    )

    # Relationships
    images: Mapped[list["ShowcaseImage"]] = relationship(
        "ShowcaseImage",
        back_populates="showcase",
        cascade="all, delete-orphan",
        order_by="ShowcaseImage.sort_order",
    )


class ShowcaseImage(Base):
    """Association table for Showcase and Image with ordering."""

    __tablename__ = "showcase_images"

    # Composite primary key
    showcase_id: Mapped[UUID] = mapped_column(
        PG_UUID(as_uuid=True),
        ForeignKey("showcases.id", ondelete="CASCADE"),
        primary_key=True,
    )
    image_id: Mapped[UUID] = mapped_column(
        PG_UUID(as_uuid=True),
        ForeignKey("images.id", ondelete="CASCADE"),
        primary_key=True,
    )

    # Sort order within the showcase
    sort_order: Mapped[int] = mapped_column(Integer, default=0, nullable=False)

    # Timestamp when added
    added_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=datetime.utcnow, nullable=False
    )

    # Relationships
    showcase: Mapped["Showcase"] = relationship("Showcase", back_populates="images")
