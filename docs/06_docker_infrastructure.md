# Docker構成・インフラ設計

## ディレクトリ構成

```text
prompt-box/
├── docker-compose.yml
├── .env.example
├── .env                      # 環境変数（gitignore）
├── README.md
│
├── backend/
│   ├── Dockerfile
│   ├── requirements.txt
│   ├── pyproject.toml
│   ├── alembic.ini
│   ├── alembic/
│   │   ├── env.py
│   │   └── versions/
│   └── app/
│       └── ...
│
├── frontend/
│   ├── Dockerfile
│   ├── nginx.conf
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   └── src/
│       └── ...
│
├── db/
│   └── init/
│       └── 01_init.sql       # DB初期化スクリプト
│
├── import/                   # 画像取り込みフォルダ（ホストマウント）
│
└── storage/                  # 画像ストレージ（ホストマウント）
```

---

## docker-compose.yml

```yaml
version: '3.8'

services:
  # ===========================================
  # Frontend (Nginx + React)
  # ===========================================
  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - "${FRONTEND_PORT:-3000}:80"
    depends_on:
      - backend
    restart: unless-stopped
    networks:
      - app-network

  # ===========================================
  # Backend (FastAPI)
  # ===========================================
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    ports:
      - "${BACKEND_PORT:-8000}:8000"
    volumes:
      - ./import:/app/import:ro          # 取り込みフォルダ（読み取り専用）
      - ./storage:/app/storage           # 画像ストレージ
    environment:
      - DATABASE_URL=postgresql+asyncpg://${DB_USER}:${DB_PASSWORD}@db:5432/${DB_NAME}
      - ADMIN_USERNAME=${ADMIN_USERNAME}
      - ADMIN_PASSWORD_HASH=${ADMIN_PASSWORD_HASH}
      - SECRET_KEY=${SECRET_KEY}
      - IMPORT_PATH=/app/import
      - STORAGE_PATH=/app/storage
      - CORS_ORIGINS=http://localhost:${FRONTEND_PORT:-3000}
    depends_on:
      db:
        condition: service_healthy
    restart: unless-stopped
    networks:
      - app-network

  # ===========================================
  # Database (PostgreSQL)
  # ===========================================
  db:
    image: postgres:16-alpine
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./db/init:/docker-entrypoint-initdb.d:ro
    environment:
      - POSTGRES_USER=${DB_USER}
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=${DB_NAME}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${DB_USER} -d ${DB_NAME}"]
      interval: 5s
      timeout: 5s
      retries: 5
    restart: unless-stopped
    # 軽量設定（1ユーザー向け）
    command: >
      postgres
      -c shared_buffers=128MB
      -c effective_cache_size=256MB
      -c work_mem=16MB
      -c maintenance_work_mem=64MB
      -c max_connections=20
      -c logging_collector=on
      -c log_statement=none
    networks:
      - app-network

networks:
  app-network:
    driver: bridge

volumes:
  postgres_data:
```

---

## 環境変数

### .env.example

```bash
# ===========================================
# Database
# ===========================================
DB_USER=comfyui
DB_PASSWORD=your_secure_password_here
DB_NAME=comfyui_gallery

# ===========================================
# Backend
# ===========================================
# セッション暗号化キー（32文字以上のランダム文字列）
SECRET_KEY=your_secret_key_here_minimum_32_characters

# 管理者認証
ADMIN_USERNAME=admin
# bcryptハッシュ化されたパスワード
# 生成コマンド: python -c "import bcrypt; print(bcrypt.hashpw(b'your_password', bcrypt.gensalt()).decode())"
ADMIN_PASSWORD_HASH=$2b$12$xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# ===========================================
# Ports
# ===========================================
FRONTEND_PORT=3000
BACKEND_PORT=8000
```

---

## Backend Dockerfile

```dockerfile
FROM python:3.11-slim

WORKDIR /app

# システム依存パッケージ
RUN apt-get update && apt-get install -y \
    libpq-dev \
    gcc \
    && rm -rf /var/lib/apt/lists/*

# Python依存パッケージ
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# アプリケーションコード
COPY . .

# 非rootユーザーで実行
RUN useradd -m appuser && chown -R appuser:appuser /app
USER appuser

# ポート公開
EXPOSE 8000

# 起動コマンド
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
```

---

## Frontend Dockerfile

```dockerfile
# ビルドステージ
FROM node:20-alpine AS builder

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

# 本番ステージ
FROM nginx:alpine

# Nginx設定
COPY nginx.conf /etc/nginx/conf.d/default.conf

# ビルド成果物をコピー
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

---

## Frontend nginx.conf

```nginx
server {
    listen 80;
    server_name localhost;
    root /usr/share/nginx/html;
    index index.html;

    # gzip圧縮
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript;

    # SPAルーティング
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API プロキシ
    location /api/ {
        proxy_pass http://backend:8000/api/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }

    # 画像ストレージ（直接配信）
    location /storage/ {
        alias /app/storage/;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }

    # 静的アセットのキャッシュ
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

**注意:** 画像配信のためfrontendコンテナにもstorageをマウントする必要がある。
docker-compose.ymlのfrontendサービスに以下を追加:

```yaml
  frontend:
    volumes:
      - ./storage:/app/storage:ro
```

---

## DB初期化スクリプト

### db/init/01_init.sql

```sql
-- 拡張機能
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- imagesテーブル（03_database_schema.md参照）
CREATE TABLE images (
    id UUID PRIMARY KEY,
    source_tool VARCHAR(20) NOT NULL,
    model_type VARCHAR(20),
    has_metadata BOOLEAN NOT NULL DEFAULT TRUE,
    original_filename VARCHAR(255) NOT NULL,
    storage_path VARCHAR(512) NOT NULL,
    thumbnail_path VARCHAR(512) NOT NULL,
    file_hash VARCHAR(64) NOT NULL UNIQUE,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    positive_prompt TEXT,
    negative_prompt TEXT,
    model_name VARCHAR(255),
    sampler_name VARCHAR(50),
    scheduler VARCHAR(50),
    steps INTEGER,
    cfg_scale NUMERIC(5,2),
    seed BIGINT,
    loras JSONB NOT NULL DEFAULT '[]'::jsonb,
    controlnets JSONB NOT NULL DEFAULT '[]'::jsonb,
    embeddings JSONB NOT NULL DEFAULT '[]'::jsonb,
    model_params JSONB NOT NULL DEFAULT '{}'::jsonb,
    workflow_extras JSONB NOT NULL DEFAULT '{}'::jsonb,
    raw_metadata JSONB,
    rating SMALLINT NOT NULL DEFAULT 0,
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
    needs_improvement BOOLEAN NOT NULL DEFAULT FALSE,
    user_tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    user_memo TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT rating_range CHECK (rating >= 0 AND rating <= 5),
    CONSTRAINT valid_dimensions CHECK (width > 0 AND height > 0),
    CONSTRAINT valid_file_size CHECK (file_size_bytes > 0)
);

-- インデックス
CREATE INDEX idx_images_source_tool ON images(source_tool);
CREATE INDEX idx_images_model_type ON images(model_type);
CREATE INDEX idx_images_model_name ON images(model_name);
CREATE INDEX idx_images_sampler_name ON images(sampler_name);
CREATE INDEX idx_images_steps ON images(steps);
CREATE INDEX idx_images_cfg_scale ON images(cfg_scale);
CREATE INDEX idx_images_rating ON images(rating);
CREATE INDEX idx_images_created_at ON images(created_at DESC);
CREATE INDEX idx_images_is_favorite ON images(is_favorite) WHERE is_favorite = TRUE;
CREATE INDEX idx_images_needs_improvement ON images(needs_improvement) WHERE needs_improvement = TRUE;
CREATE INDEX idx_images_deleted ON images(deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX idx_images_model_name_trgm ON images USING gin(model_name gin_trgm_ops);
CREATE INDEX idx_images_positive_prompt_fts ON images USING gin(to_tsvector('english', COALESCE(positive_prompt, '')));
CREATE INDEX idx_images_negative_prompt_fts ON images USING gin(to_tsvector('english', COALESCE(negative_prompt, '')));
CREATE INDEX idx_images_loras ON images USING gin(loras jsonb_path_ops);
CREATE INDEX idx_images_controlnets ON images USING gin(controlnets jsonb_path_ops);
CREATE INDEX idx_images_user_tags ON images USING gin(user_tags jsonb_path_ops);
CREATE INDEX idx_images_model_params ON images USING gin(model_params jsonb_path_ops);
CREATE INDEX idx_images_list_default ON images(created_at DESC) WHERE deleted_at IS NULL;

-- updated_at自動更新トリガー
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_images_updated_at
    BEFORE UPDATE ON images
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

---

## 起動手順

```bash
# 1. 環境変数ファイルを作成
cp .env.example .env
# .envを編集してパスワード等を設定

# 2. パスワードハッシュを生成
python -c "import bcrypt; print(bcrypt.hashpw(b'your_password', bcrypt.gensalt()).decode())"
# 出力を.envのADMIN_PASSWORD_HASHに設定

# 3. ディレクトリ作成
mkdir -p import storage

# 4. ビルド＆起動
docker compose up -d --build

# 5. ログ確認
docker compose logs -f

# 6. 停止
docker compose down

# 7. データ含めて完全削除
docker compose down -v
```

---

## 開発モード

開発時はホットリロードを有効にする。

### docker-compose.override.yml（開発用）

```yaml
version: '3.8'

services:
  backend:
    volumes:
      - ./backend/app:/app/app:ro
    command: uvicorn app.main:app --host 0.0.0.0 --port 8000 --reload
    environment:
      - DEBUG=1

  frontend:
    build:
      target: builder
    ports:
      - "${FRONTEND_PORT:-3000}:5173"
    volumes:
      - ./frontend/src:/app/src
    command: npm run dev -- --host 0.0.0.0
```

```bash
# 開発モードで起動
docker compose -f docker-compose.yml -f docker-compose.override.yml up
```
