from pydantic_settings import BaseSettings
from functools import lru_cache


class Settings(BaseSettings):
    # Database
    database_url: str = "postgresql+asyncpg://comfyui:password@db:5432/comfyui_gallery"

    # Auth
    admin_username: str = "admin"
    admin_password_hash: str = ""
    secret_key: str = "your_secret_key_here_minimum_32_characters"
    session_expire_hours: int = 24 * 7  # 1 week

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
