#!/bin/bash
set -euo pipefail

DOCKER_USER="urusy7"
DEFAULT_PLATFORM="linux/amd64,linux/arm64"

# バージョンタグ決定:
#   1. 引数で明示指定されたもの (e.g. ./deploy.sh v1.0.0)
#   2. 直近の git tag (--exact-match → fall back to --abbrev=0)
#   3. git short SHA (タグが無いとき)
if [ "$#" -ge 1 ]; then
    VERSION_TAG="$1"
else
    VERSION_TAG="$(git describe --tags --exact-match 2>/dev/null \
        || git describe --tags --abbrev=0 2>/dev/null \
        || git rev-parse --short HEAD 2>/dev/null \
        || echo "dev")"
fi

PLATFORM="${PLATFORM:-${DEFAULT_PLATFORM}}"

FRONTEND_BASE="${DOCKER_USER}/promptbox-frontend"
BACKEND_BASE="${DOCKER_USER}/promptbox-backend"

# プロジェクトルートに移動
cd "$(dirname "$0")/.."

echo "========================================="
echo "  Prompt Box - Build & Push to Docker Hub"
echo "========================================="
echo ""
echo "Platform:      ${PLATFORM}"
echo "Version tag:   ${VERSION_TAG}"
echo "Frontend:      ${FRONTEND_BASE}:${VERSION_TAG} (+ :latest)"
echo "Backend:       ${BACKEND_BASE}:${VERSION_TAG} (+ :latest)"
echo ""

# buildx builder 確認（存在しなければ作成）
if ! docker buildx inspect promptbox-builder >/dev/null 2>&1; then
    docker buildx create --name promptbox-builder --use >/dev/null
else
    docker buildx use promptbox-builder >/dev/null
fi

# フロントエンドのビルド＆プッシュ（バージョンタグ + latest 両方）
echo "-----------------------------------------"
echo "[1/2] Building & pushing frontend..."
echo "-----------------------------------------"
docker buildx build --platform "${PLATFORM}" \
    -t "${FRONTEND_BASE}:${VERSION_TAG}" \
    -t "${FRONTEND_BASE}:latest" \
    --push ./frontend
echo "Frontend pushed successfully."
echo ""

# バックエンドのビルド＆プッシュ
echo "-----------------------------------------"
echo "[2/2] Building & pushing backend..."
echo "-----------------------------------------"
docker buildx build --platform "${PLATFORM}" \
    -t "${BACKEND_BASE}:${VERSION_TAG}" \
    -t "${BACKEND_BASE}:latest" \
    --push ./backend
echo "Backend pushed successfully."
echo ""

echo "========================================="
echo "  Pushed: ${VERSION_TAG} (and :latest)"
echo "========================================="
echo ""
echo "本番で特定版に固定するには docker-compose.prod.yml の image タグを書き換え:"
echo "  image: ${FRONTEND_BASE}:${VERSION_TAG}"
echo "  image: ${BACKEND_BASE}:${VERSION_TAG}"
