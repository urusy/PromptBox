from fastapi import APIRouter, HTTPException, Response, status

from app.config import get_settings
from app.schemas.auth import LoginRequest, LoginResponse
from app.schemas.common import MessageResponse
from app.services.auth_service import AuthService
from app.api.deps import CurrentUser

router = APIRouter(prefix="/auth", tags=["auth"])


@router.post("/login", response_model=LoginResponse)
async def login(request: LoginRequest, response: Response) -> LoginResponse:
    """Login with username and password."""
    auth_service = AuthService()
    settings = get_settings()

    if not auth_service.verify_password(request.username, request.password):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid username or password",
        )

    session_token = auth_service.create_session(request.username)

    # In debug mode, allow HTTP cookies for local development
    response.set_cookie(
        key="session",
        value=session_token,
        httponly=True,
        secure=not settings.debug,
        samesite="lax" if settings.debug else "strict",
        max_age=60 * 60 * 24 * 7,  # 1 week
    )

    return LoginResponse(message="Login successful", username=request.username)


@router.post("/logout", response_model=MessageResponse)
async def logout(response: Response, _: CurrentUser) -> MessageResponse:
    """Logout and clear session."""
    response.delete_cookie(key="session")
    return MessageResponse(message="Logout successful")


@router.get("/me")
async def get_me(username: CurrentUser) -> dict[str, str]:
    """Get current user info."""
    return {"username": username}
