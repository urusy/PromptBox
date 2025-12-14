"""Pytest configuration and shared fixtures."""

import asyncio
import os
import tempfile
from collections.abc import AsyncGenerator, Generator
from pathlib import Path
from typing import Any

import pytest
from httpx import ASGITransport, AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.pool import StaticPool

# Set test environment before importing app modules
os.environ["TESTING"] = "1"
os.environ["SECRET_KEY"] = "test-secret-key-for-testing-only-32chars"
os.environ["ADMIN_USERNAME"] = "testadmin"
os.environ["ADMIN_PASSWORD_HASH"] = (
    "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.S4kiYCb/Xce9BO"  # "testpass"
)
os.environ["DATABASE_URL"] = "sqlite+aiosqlite:///:memory:"

from app.api.deps import get_db
from app.database import Base
from app.main import app

# Test database URL (in-memory SQLite for tests)
TEST_DATABASE_URL = "sqlite+aiosqlite:///:memory:"


@pytest.fixture(scope="session")
def event_loop() -> Generator[asyncio.AbstractEventLoop, None, None]:
    """Create an event loop for the test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


@pytest.fixture
async def async_engine():
    """Create an async engine for testing."""
    engine = create_async_engine(
        TEST_DATABASE_URL,
        echo=False,
        poolclass=StaticPool,
        connect_args={"check_same_thread": False},
    )

    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)

    yield engine

    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.drop_all)

    await engine.dispose()


@pytest.fixture
async def db_session(async_engine) -> AsyncGenerator[AsyncSession, None]:
    """Create a database session for testing."""
    session_factory = async_sessionmaker(
        async_engine,
        class_=AsyncSession,
        expire_on_commit=False,
    )

    async with session_factory() as session:
        yield session
        await session.rollback()


@pytest.fixture
async def client(db_session: AsyncSession) -> AsyncGenerator[AsyncClient, None]:
    """Create a test client with database session override."""

    async def override_get_db() -> AsyncGenerator[AsyncSession, None]:
        yield db_session

    app.dependency_overrides[get_db] = override_get_db

    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as ac:
        yield ac

    app.dependency_overrides.clear()


@pytest.fixture
def temp_dir() -> Generator[Path, None, None]:
    """Create a temporary directory for file tests."""
    with tempfile.TemporaryDirectory() as tmpdir:
        yield Path(tmpdir)


@pytest.fixture
def sample_png_info_comfyui() -> dict[str, Any]:
    """Sample ComfyUI metadata for testing."""
    return {
        "prompt": """{
            "1": {
                "class_type": "CheckpointLoaderSimple",
                "inputs": {
                    "ckpt_name": "sd_xl_base_1.0.safetensors"
                }
            },
            "2": {
                "class_type": "CLIPTextEncode",
                "inputs": {
                    "text": "beautiful sunset over ocean, masterpiece",
                    "clip": ["1", 1]
                }
            },
            "3": {
                "class_type": "CLIPTextEncode",
                "inputs": {
                    "text": "ugly, blurry",
                    "clip": ["1", 1]
                }
            },
            "4": {
                "class_type": "KSampler",
                "inputs": {
                    "seed": 12345,
                    "steps": 20,
                    "cfg": 7.5,
                    "sampler_name": "euler",
                    "scheduler": "normal",
                    "positive": ["2", 0],
                    "negative": ["3", 0],
                    "model": ["1", 0]
                }
            }
        }""",
        "workflow": '{"nodes": [1, 2, 3, 4], "version": "0.1.0"}',
    }


@pytest.fixture
def sample_png_info_a1111() -> dict[str, Any]:
    """Sample A1111 metadata for testing."""
    return {
        "parameters": """beautiful sunset over ocean, masterpiece, <lora:detail_slider:0.8>
Negative prompt: ugly, blurry
Steps: 30, Sampler: DPM++ 2M Karras, CFG scale: 7.5, Seed: 67890, Size: 1024x1024, Model: animagineXL_v30, Model hash: abc123, Clip skip: 2"""
    }


@pytest.fixture
def sample_png_info_novelai() -> dict[str, Any]:
    """Sample NovelAI metadata for testing."""
    return {
        "Software": "NovelAI",
        "Comment": """{
            "prompt": "beautiful sunset over ocean, masterpiece",
            "uc": "ugly, blurry",
            "steps": 28,
            "sampler": "k_euler_ancestral",
            "seed": 111222,
            "strength": 0.7,
            "noise": 0.2,
            "scale": 11,
            "n_samples": 1,
            "ucPreset": 0
        }""",
    }


@pytest.fixture
def sample_png_info_empty() -> dict[str, Any]:
    """Empty metadata for testing unknown source."""
    return {}
