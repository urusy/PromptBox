from fastapi import APIRouter

from app.api.endpoints import auth, batch, duplicates, export, health, images

api_router = APIRouter(prefix="/api")

api_router.include_router(health.router)
api_router.include_router(auth.router)
api_router.include_router(images.router)
api_router.include_router(batch.router)
api_router.include_router(export.router)
api_router.include_router(duplicates.router)
