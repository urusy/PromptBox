from datetime import datetime, timedelta
from typing import Any

import bcrypt
from jose import JWTError, jwt

from app.config import get_settings


class AuthService:
    def __init__(self) -> None:
        self.settings = get_settings()

    def verify_password(self, username: str, password: str) -> bool:
        """Verify username and password against stored credentials."""
        if username != self.settings.admin_username:
            return False

        if not self.settings.admin_password_hash:
            return False

        try:
            return bcrypt.checkpw(
                password.encode("utf-8"),
                self.settings.admin_password_hash.encode("utf-8"),
            )
        except Exception:
            return False

    def create_session(self, username: str) -> str:
        """Create a JWT session token."""
        expire = datetime.utcnow() + timedelta(hours=self.settings.session_expire_hours)

        payload: dict[str, Any] = {
            "sub": username,
            "exp": expire,
            "iat": datetime.utcnow(),
        }

        return jwt.encode(payload, self.settings.secret_key, algorithm="HS256")

    def verify_session(self, token: str) -> str | None:
        """Verify a session token and return the username if valid."""
        try:
            payload = jwt.decode(token, self.settings.secret_key, algorithms=["HS256"])
            username: str | None = payload.get("sub")
            return username
        except JWTError:
            return None
