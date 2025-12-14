from collections.abc import AsyncGenerator
from contextlib import asynccontextmanager
from pathlib import Path

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from loguru import logger

from app.api.router import api_router
from app.config import get_settings
from app.workers import ImageWatcher

settings = get_settings()

# Global watcher instance
_watcher: ImageWatcher | None = None


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """Application lifespan handler."""
    global _watcher

    logger.info("Starting application...")
    logger.info(f"Debug mode: {settings.debug}")

    # Start the image watcher
    _watcher = ImageWatcher()
    await _watcher.process_existing()  # Process any files already in import folder
    _watcher.start()

    yield

    # Stop the watcher
    if _watcher:
        _watcher.stop()

    logger.info("Shutting down application...")


app = FastAPI(
    title="Prompt Box API",
    description="API for managing AI-generated images",
    version="0.1.0",
    lifespan=lifespan,
)

# CORS middleware
origins = [origin.strip() for origin in settings.cors_origins.split(",")]
app.add_middleware(
    CORSMiddleware,
    allow_origins=origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include API router
app.include_router(api_router)

# Mount static files for storage (images, thumbnails)
storage_path = Path(settings.storage_path)
if storage_path.exists():
    app.mount("/storage", StaticFiles(directory=str(storage_path)), name="storage")
    logger.info(f"Mounted storage directory: {storage_path}")
else:
    logger.warning(f"Storage path does not exist: {storage_path}")


@app.get("/")
async def root() -> dict[str, str]:
    """Root endpoint."""
    return {"message": "Prompt Box API", "version": "0.1.0"}
