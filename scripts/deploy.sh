#!/bin/bash
set -e

DOCKER_USER="urusy7"
FRONTEND_IMAGE="${DOCKER_USER}/promptbox-frontend:latest"
BACKEND_IMAGE="${DOCKER_USER}/promptbox-backend:latest"
PLATFORM="linux/amd64"

# プロジェクトルートに移動
cd "$(dirname "$0")/.."

echo "========================================="
echo "  Prompt Box - Build & Push to Docker Hub"
echo "========================================="
echo ""
echo "Platform: ${PLATFORM}"
echo "Frontend: ${FRONTEND_IMAGE}"
echo "Backend:  ${BACKEND_IMAGE}"
echo ""

# フロントエンドのビルド＆プッシュ
echo "-----------------------------------------"
echo "[1/2] Building & pushing frontend..."
echo "-----------------------------------------"
docker buildx build --platform "${PLATFORM}" \
  -t "${FRONTEND_IMAGE}" --push ./frontend
echo "Frontend pushed successfully."
echo ""

# バックエンドのビルド＆プッシュ
echo "-----------------------------------------"
echo "[2/2] Building & pushing backend..."
echo "-----------------------------------------"
docker buildx build --platform "${PLATFORM}" \
  -t "${BACKEND_IMAGE}" --push ./backend
echo "Backend pushed successfully."
echo ""

echo "========================================="
echo "  All images pushed to Docker Hub!"
echo "========================================="
