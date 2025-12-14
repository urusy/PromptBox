"""Integration tests for authentication endpoints."""

import pytest
from httpx import AsyncClient


@pytest.mark.integration
class TestAuthEndpoints:
    """Integration tests for authentication endpoints."""

    async def test_login_success(self, client: AsyncClient) -> None:
        """Test successful login."""
        response = await client.post(
            "/api/auth/login",
            json={"username": "testadmin", "password": "testpass"},
        )

        assert response.status_code == 200
        data = response.json()
        assert data["message"] == "Login successful"
        assert data["username"] == "testadmin"
        assert "session" in response.cookies

    async def test_login_wrong_username(self, client: AsyncClient) -> None:
        """Test login with wrong username."""
        response = await client.post(
            "/api/auth/login",
            json={"username": "wronguser", "password": "testpass"},
        )

        assert response.status_code == 401
        data = response.json()
        assert "Invalid" in data["detail"]

    async def test_login_wrong_password(self, client: AsyncClient) -> None:
        """Test login with wrong password."""
        response = await client.post(
            "/api/auth/login",
            json={"username": "testadmin", "password": "wrongpass"},
        )

        assert response.status_code == 401

    async def test_login_empty_credentials(self, client: AsyncClient) -> None:
        """Test login with empty credentials."""
        response = await client.post(
            "/api/auth/login",
            json={"username": "", "password": ""},
        )

        assert response.status_code == 401

    async def test_get_me_authenticated(self, client: AsyncClient) -> None:
        """Test getting current user when authenticated."""
        # First login
        login_response = await client.post(
            "/api/auth/login",
            json={"username": "testadmin", "password": "testpass"},
        )
        assert login_response.status_code == 200

        # Get session cookie
        session_cookie = login_response.cookies.get("session")
        assert session_cookie is not None

        # Get current user
        response = await client.get(
            "/api/auth/me",
            cookies={"session": session_cookie},
        )

        assert response.status_code == 200
        data = response.json()
        assert data["username"] == "testadmin"

    async def test_get_me_unauthenticated(self, client: AsyncClient) -> None:
        """Test getting current user when not authenticated."""
        response = await client.get("/api/auth/me")

        assert response.status_code == 401

    async def test_logout(self, client: AsyncClient) -> None:
        """Test logout clears session."""
        # First login
        login_response = await client.post(
            "/api/auth/login",
            json={"username": "testadmin", "password": "testpass"},
        )
        session_cookie = login_response.cookies.get("session")

        # Logout
        response = await client.post(
            "/api/auth/logout",
            cookies={"session": session_cookie},
        )

        assert response.status_code == 200
        data = response.json()
        assert data["message"] == "Logout successful"

    async def test_logout_unauthenticated(self, client: AsyncClient) -> None:
        """Test logout when not authenticated."""
        response = await client.post("/api/auth/logout")

        assert response.status_code == 401
