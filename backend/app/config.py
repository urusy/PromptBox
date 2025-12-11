import secrets
from functools import lru_cache

from pydantic import field_validator
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    # Database
    database_url: str = "postgresql+asyncpg://comfyui:password@db:5432/comfyui_gallery"

    # Auth
    admin_username: str = "admin"
    admin_password_hash: str = ""
    secret_key: str = ""
    session_expire_hours: int = 24 * 7  # 1 week

    @field_validator("secret_key", mode="after")
    @classmethod
    def validate_secret_key(cls, v: str, info) -> str:
        """Generate secure secret key if not provided."""
        if not v:
            # Generate a random secret key for development
            # In production, this should be set via environment variable
            return secrets.token_urlsafe(32)
        if len(v) < 32:
            raise ValueError("secret_key must be at least 32 characters")
        return v

    # Paths
    import_path: str = "/app/import"
    storage_path: str = "/app/storage"

    # CORS
    cors_origins: str = "http://localhost:3000"

    # Debug
    debug: bool = False

    # Thumbnail
    thumbnail_size: int = 300
    thumbnail_quality: int = 85

    class Config:
        env_file = ".env"
        case_sensitive = False


@lru_cache
def get_settings() -> Settings:
    return Settings()
