"""Tests for AuthService."""

from datetime import datetime, timedelta

import pytest
from jose import jwt

from app.services.auth_service import AuthService


class TestAuthService:
    """Test cases for AuthService."""

    @pytest.fixture
    def auth_service(self) -> AuthService:
        """Create an AuthService instance with test settings."""
        return AuthService()

    def test_verify_password_correct(self, auth_service: AuthService) -> None:
        """Test password verification with correct credentials."""
        # The conftest sets up test credentials
        result = auth_service.verify_password("testadmin", "testpass")
        assert result is True

    def test_verify_password_wrong_username(self, auth_service: AuthService) -> None:
        """Test password verification with wrong username."""
        result = auth_service.verify_password("wronguser", "testpass")
        assert result is False

    def test_verify_password_wrong_password(self, auth_service: AuthService) -> None:
        """Test password verification with wrong password."""
        result = auth_service.verify_password("testadmin", "wrongpass")
        assert result is False

    def test_verify_password_empty_password(self, auth_service: AuthService) -> None:
        """Test password verification with empty password."""
        result = auth_service.verify_password("testadmin", "")
        assert result is False

    def test_create_session(self, auth_service: AuthService) -> None:
        """Test session token creation."""
        token = auth_service.create_session("testadmin")

        assert isinstance(token, str)
        assert len(token) > 0

        # Decode and verify the token structure
        payload = jwt.decode(
            token,
            auth_service.settings.secret_key,
            algorithms=["HS256"],
        )
        assert payload["sub"] == "testadmin"
        assert "exp" in payload
        assert "iat" in payload

    def test_verify_session_valid(self, auth_service: AuthService) -> None:
        """Test session verification with valid token."""
        token = auth_service.create_session("testadmin")
        result = auth_service.verify_session(token)

        assert result == "testadmin"

    def test_verify_session_invalid_token(self, auth_service: AuthService) -> None:
        """Test session verification with invalid token."""
        result = auth_service.verify_session("invalid.token.here")

        assert result is None

    def test_verify_session_expired_token(self, auth_service: AuthService) -> None:
        """Test session verification with expired token."""
        # Create an expired token manually
        expired_payload = {
            "sub": "testadmin",
            "exp": datetime.utcnow() - timedelta(hours=1),
            "iat": datetime.utcnow() - timedelta(hours=2),
        }
        expired_token = jwt.encode(
            expired_payload,
            auth_service.settings.secret_key,
            algorithm="HS256",
        )

        result = auth_service.verify_session(expired_token)

        assert result is None

    def test_verify_session_tampered_token(self, auth_service: AuthService) -> None:
        """Test session verification with tampered token."""
        token = auth_service.create_session("testadmin")

        # Tamper with the token
        parts = token.split(".")
        parts[1] = parts[1] + "tampered"
        tampered_token = ".".join(parts)

        result = auth_service.verify_session(tampered_token)

        assert result is None

    def test_verify_session_wrong_secret(self, auth_service: AuthService) -> None:
        """Test session verification with token signed by wrong secret."""
        wrong_secret_payload = {
            "sub": "testadmin",
            "exp": datetime.utcnow() + timedelta(hours=1),
            "iat": datetime.utcnow(),
        }
        wrong_secret_token = jwt.encode(
            wrong_secret_payload,
            "wrong-secret-key-32-characters-here",
            algorithm="HS256",
        )

        result = auth_service.verify_session(wrong_secret_token)

        assert result is None
