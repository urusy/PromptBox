"""Integration tests for health endpoints."""

import pytest
from httpx import AsyncClient


@pytest.mark.integration
class TestHealthEndpoints:
    """Integration tests for health check endpoints."""

    async def test_health_check(self, client: AsyncClient) -> None:
        """Test basic health check endpoint."""
        response = await client.get("/api/health")

        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"

    async def test_db_health_check(self, client: AsyncClient) -> None:
        """Test database health check endpoint."""
        response = await client.get("/api/health/db")

        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"
        assert data["database"] == "connected"
