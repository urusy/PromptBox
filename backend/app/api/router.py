from fastapi import APIRouter

from app.api.endpoints import (
    auth,
    batch,
    duplicates,
    export,
    health,
    images,
    search_presets,
    smart_folders,
    stats,
    tags,
)

api_router = APIRouter(prefix="/api")

api_router.include_router(health.router)
api_router.include_router(auth.router)
api_router.include_router(images.router)
api_router.include_router(batch.router)
api_router.include_router(export.router)
api_router.include_router(duplicates.router)
api_router.include_router(search_presets.router)
api_router.include_router(smart_folders.router)
api_router.include_router(stats.router)
api_router.include_router(tags.router)
