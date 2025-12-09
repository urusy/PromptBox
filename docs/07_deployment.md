# デプロイメントガイド

## 概要

開発マシン（Mac）でDockerイメージをビルドし、Docker Hub経由でNAS本番環境にデプロイする。

---

## 本番イメージのビルドとプッシュ

開発マシン（Mac）からDocker Hubにイメージをプッシュする。
NASはamd64アーキテクチャのため、`buildx`でクロスビルドが必要。

### 前提条件

```bash
# Docker Hubにログイン（初回のみ）
docker login -u urusy7
```

### ビルド＆プッシュ

```bash
# フロントエンドのビルド＆プッシュ
docker buildx build --platform linux/amd64 \
  -t urusy7/promptbox-frontend:latest --push ./frontend

# バックエンドのビルド＆プッシュ
docker buildx build --platform linux/amd64 \
  -t urusy7/promptbox-backend:latest --push ./backend
```

### 一括実行

```bash
docker buildx build --platform linux/amd64 -t urusy7/promptbox-frontend:latest --push ./frontend && \
docker buildx build --platform linux/amd64 -t urusy7/promptbox-backend:latest --push ./backend
```

---

## NAS本番環境

### 環境情報

| 項目 | 値 |
|------|-----|
| パス | `/volume1/docker/prompt_box/` |
| 構成ファイル | `docker-compose.yaml` |
| 実行権限 | sudo必要 |

### ディレクトリ構成

```text
/volume1/docker/prompt_box/
├── docker-compose.yaml      # docker-compose.prod.ymlの内容
├── .env                     # 環境変数
├── db/
│   └── init/
│       └── 01_init.sql      # DB初期化スクリプト
├── import/                  # 画像取り込みフォルダ
└── storage/                 # 画像ストレージ
```

---

## 初回セットアップ（NAS）

```bash
cd /volume1/docker/prompt_box/

# ディレクトリのパーミッション設定
sudo chmod -R 777 storage
sudo chmod -R 777 import

# 起動
sudo docker compose -f docker-compose.yaml up -d
```

### DB初期化（初回または再初期化時）

```bash
sudo docker compose -f docker-compose.yaml exec db \
  psql -U promptbox -d promptbox -f /docker-entrypoint-initdb.d/01_init.sql
```

---

## アップデート手順（NAS）

開発マシンでイメージをプッシュ後、NASで以下を実行。

### 標準アップデート

```bash
cd /volume1/docker/prompt_box/

# コンテナを停止
sudo docker compose -f docker-compose.yaml down

# 最新イメージをプル＆再起動
sudo docker compose -f docker-compose.yaml up -d --force-recreate
```

### クイックアップデート（ダウンタイム最小化）

```bash
cd /volume1/docker/prompt_box/

# 最新イメージをプル
sudo docker compose -f docker-compose.yaml pull

# コンテナを再作成
sudo docker compose -f docker-compose.yaml up -d --force-recreate
```

---

## トラブルシューティング

### コンテナ状態確認

```bash
sudo docker compose -f docker-compose.yaml ps
```

### ログ確認

```bash
# 全コンテナ
sudo docker compose -f docker-compose.yaml logs -f

# 個別コンテナ
sudo docker compose -f docker-compose.yaml logs backend
sudo docker compose -f docker-compose.yaml logs frontend
```

### DB接続確認

```bash
sudo docker compose -f docker-compose.yaml exec db pg_isready -h 0.0.0.0 -p 5432
```

### ポート確認

```bash
sudo netstat -tlnp | grep 5432   # PostgreSQL
sudo netstat -tlnp | grep 3000   # Frontend
sudo netstat -tlnp | grep 8000   # Backend
```
