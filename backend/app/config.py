import secrets
from functools import lru_cache

from loguru import logger
from pydantic import model_validator
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    # Database
    database_url: str = "postgresql+asyncpg://comfyui:password@db:5432/comfyui_gallery"

    # Auth
    admin_username: str = "admin"
    admin_password_hash: str = ""
    secret_key: str = ""
    session_expire_hours: int = 24 * 7  # 1 week

    # Paths
    import_path: str = "/app/import"
    storage_path: str = "/app/storage"

    # CORS
    cors_origins: str = "http://localhost:3000"

    # Gelbooru API
    gelbooru_api_key: str = ""
    gelbooru_user_id: str = ""

    # Debug
    debug: bool = False

    # Thumbnail
    thumbnail_size: int = 300
    thumbnail_quality: int = 85

    @model_validator(mode="after")
    def _validate_secret_key(self) -> "Settings":
        """Require an explicit SECRET_KEY in production; generate ephemeral one in dev."""
        if not self.secret_key:
            if not self.debug:
                raise ValueError(
                    "SECRET_KEY must be set explicitly in production (debug=False). "
                    "Set the SECRET_KEY environment variable to a random string of at "
                    "least 32 characters."
                )
            # Development convenience: generate an ephemeral key. Sessions will be
            # invalidated on restart — acceptable for local dev only.
            logger.warning(
                "SECRET_KEY not set; generating an ephemeral key for development. "
                "Sessions will be invalidated on every restart."
            )
            self.secret_key = secrets.token_urlsafe(32)
        elif len(self.secret_key) < 32:
            raise ValueError("secret_key must be at least 32 characters")
        return self

    class Config:
        env_file = ".env"
        case_sensitive = False


@lru_cache
def get_settings() -> Settings:
    return Settings()
