from typing import Annotated

from fastapi import Cookie, Depends, HTTPException, status
from sqlalchemy.ext.asyncio import AsyncSession

from app.database import get_db
from app.services.auth_service import AuthService

DbSession = Annotated[AsyncSession, Depends(get_db)]


async def get_current_user(
    session_token: str | None = Cookie(None, alias="session"),
) -> str:
    """Get the current authenticated user from session cookie."""
    if not session_token:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Not authenticated",
        )

    auth_service = AuthService()
    username = auth_service.verify_session(session_token)

    if not username:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid or expired session",
        )

    return username


CurrentUser = Annotated[str, Depends(get_current_user)]
