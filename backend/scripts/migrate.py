"""Idempotent DB migration runner.

Behaviour:
- If the alembic_version table is missing but images table exists
  (i.e. schema was created by db/init/*.sql on first container boot),
  stamp the baseline revision so `alembic upgrade head` becomes a no-op.
- Otherwise run `alembic upgrade head` normally.
"""
from __future__ import annotations

import asyncio
import sys

from alembic import command
from alembic.config import Config
from loguru import logger
from sqlalchemy import text
from sqlalchemy.ext.asyncio import create_async_engine

from app.config import get_settings

BASELINE_REVISION = "0001_baseline"


async def _table_exists(engine, qualified_name: str) -> bool:
    async with engine.connect() as conn:
        result = await conn.execute(
            text("SELECT to_regclass(:name) IS NOT NULL").bindparams(name=qualified_name)
        )
        return bool(result.scalar())


async def _inspect_db() -> tuple[bool, bool]:
    settings = get_settings()
    engine = create_async_engine(settings.database_url)
    try:
        alembic_exists = await _table_exists(engine, "public.alembic_version")
        images_exists = await _table_exists(engine, "public.images")
    finally:
        await engine.dispose()
    return alembic_exists, images_exists


def _run_migrations() -> None:
    alembic_exists, images_exists = asyncio.run(_inspect_db())

    cfg = Config("alembic.ini")

    if not alembic_exists and images_exists:
        logger.info("Baseline stamp required: images table exists but alembic_version is missing")
        command.stamp(cfg, BASELINE_REVISION)

    logger.info("Running alembic upgrade head")
    command.upgrade(cfg, "head")


if __name__ == "__main__":
    try:
        _run_migrations()
    except Exception as e:  # noqa: BLE001
        logger.error(f"Migration failed: {e}")
        sys.exit(1)
