# 技術スタック・アーキテクチャ設計

## 技術スタック

### バックエンド

| 技術 | バージョン | 用途 |
|------|-----------|------|
| Python | 3.11+ | ランタイム |
| FastAPI | 最新 | Webフレームワーク |
| SQLAlchemy | 2.0+ | ORM（非同期対応） |
| asyncpg | 最新 | PostgreSQL非同期ドライバ |
| Pillow | 最新 | 画像処理、サムネイル生成 |
| watchdog | 最新 | フォルダ監視 |
| uuid-utils | 最新 | UUID v7生成 |
| bcrypt | 最新 | パスワードハッシュ |
| python-multipart | 最新 | ファイルアップロード |
| aiofiles | 最新 | 非同期ファイル操作 |

### フロントエンド

| 技術 | バージョン | 用途 |
|------|-----------|------|
| React | 18 | UIフレームワーク |
| TypeScript | 5+ | 型安全 |
| TailwindCSS | 3 | スタイリング |
| TanStack Query | 5 | サーバー状態管理 |
| React Router | 6 | ルーティング |
| Axios | 最新 | HTTPクライアント |
| Zustand | 最新 | クライアント状態管理 |
| react-hot-toast | 最新 | 通知 |

### インフラ

| 技術 | バージョン | 用途 |
|------|-----------|------|
| Docker | 最新 | コンテナ化 |
| Docker Compose | 最新 | オーケストレーション |
| PostgreSQL | 16 | データベース |
| Nginx | 最新 | リバースプロキシ（フロントエンド配信） |

---

## アーキテクチャ概要

```text
┌─────────────────────────────────────────────────────────────────┐
│                         Docker Network                          │
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐ │
│  │  Frontend   │    │   Backend   │    │     PostgreSQL      │ │
│  │   (Nginx)   │───▶│  (FastAPI)  │───▶│                     │ │
│  │   :3000     │    │   :8000     │    │       :5432         │ │
│  └─────────────┘    └──────┬──────┘    └─────────────────────┘ │
│                            │                                    │
│                            ▼                                    │
│                    ┌───────────────┐                           │
│                    │   Volumes     │                           │
│                    │  - /import    │◀── 監視フォルダ            │
│                    │  - /storage   │◀── 画像ストレージ          │
│                    └───────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
```

---

## バックエンドアーキテクチャ

### レイヤー構成

```text
backend/
├── app/
│   ├── main.py              # FastAPIエントリーポイント
│   ├── config.py            # 設定管理
│   ├── database.py          # DB接続設定
│   │
│   ├── api/                  # APIレイヤー
│   │   ├── __init__.py
│   │   ├── deps.py          # 依存性注入
│   │   ├── router.py        # ルーター集約
│   │   └── endpoints/
│   │       ├── auth.py      # 認証API
│   │       ├── images.py    # 画像API
│   │       ├── tags.py      # タグAPI
│   │       └── export.py    # エクスポートAPI
│   │
│   ├── services/            # ビジネスロジック
│   │   ├── __init__.py
│   │   ├── image_service.py
│   │   ├── tag_service.py
│   │   ├── export_service.py
│   │   └── auth_service.py
│   │
│   ├── repositories/        # データアクセス
│   │   ├── __init__.py
│   │   ├── image_repository.py
│   │   └── tag_repository.py
│   │
│   ├── models/              # SQLAlchemyモデル
│   │   ├── __init__.py
│   │   └── image.py
│   │
│   ├── schemas/             # Pydanticスキーマ
│   │   ├── __init__.py
│   │   ├── image.py
│   │   ├── tag.py
│   │   ├── auth.py
│   │   └── common.py
│   │
│   ├── parsers/             # メタデータパーサー
│   │   ├── __init__.py
│   │   ├── base.py          # 基底クラス
│   │   ├── comfyui.py       # ComfyUIパーサー
│   │   ├── a1111.py         # A1111パーサー
│   │   ├── factory.py       # パーサーファクトリ
│   │   └── model_detector.py # モデルタイプ検出
│   │
│   ├── workers/             # バックグラウンドワーカー
│   │   ├── __init__.py
│   │   └── watcher.py       # フォルダ監視
│   │
│   └── utils/               # ユーティリティ
│       ├── __init__.py
│       ├── image_utils.py   # 画像処理
│       ├── file_utils.py    # ファイル操作
│       └── hash_utils.py    # ハッシュ計算
│
├── alembic/                 # DBマイグレーション
│   ├── versions/
│   └── env.py
│
├── tests/                   # テスト
│   ├── conftest.py
│   ├── test_api/
│   ├── test_services/
│   └── test_parsers/
│
├── Dockerfile
├── requirements.txt
├── alembic.ini
└── pyproject.toml
```

### 依存関係の流れ

```text
API Layer (endpoints)
    ↓ 依存性注入
Service Layer (services)
    ↓
Repository Layer (repositories)
    ↓
Database (models)
```

---

## フロントエンドアーキテクチャ

### ディレクトリ構成

```text
frontend/
├── src/
│   ├── main.tsx             # エントリーポイント
│   ├── App.tsx              # ルートコンポーネント
│   ├── vite-env.d.ts
│   │
│   ├── api/                 # API通信
│   │   ├── client.ts        # Axiosクライアント
│   │   ├── images.ts        # 画像API
│   │   ├── tags.ts          # タグAPI
│   │   ├── auth.ts          # 認証API
│   │   └── types.ts         # APIレスポンス型
│   │
│   ├── components/          # UIコンポーネント
│   │   ├── common/          # 共通コンポーネント
│   │   │   ├── Button.tsx
│   │   │   ├── Input.tsx
│   │   │   ├── Modal.tsx
│   │   │   ├── Pagination.tsx
│   │   │   ├── StarRating.tsx
│   │   │   └── TagChip.tsx
│   │   │
│   │   ├── layout/          # レイアウト
│   │   │   ├── Header.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   └── MainLayout.tsx
│   │   │
│   │   ├── gallery/         # ギャラリー関連
│   │   │   ├── ImageGrid.tsx
│   │   │   ├── ImageCard.tsx
│   │   │   ├── SearchForm.tsx
│   │   │   ├── SelectionToolbar.tsx
│   │   │   └── Slideshow.tsx
│   │   │
│   │   └── detail/          # 詳細画面関連
│   │       ├── ImageViewer.tsx
│   │       ├── MetadataPanel.tsx
│   │       ├── TagEditor.tsx
│   │       └── MemoEditor.tsx
│   │
│   ├── pages/               # ページコンポーネント
│   │   ├── LoginPage.tsx
│   │   ├── GalleryPage.tsx
│   │   ├── DetailPage.tsx
│   │   └── TrashPage.tsx
│   │
│   ├── hooks/               # カスタムフック
│   │   ├── useImages.ts
│   │   ├── useTags.ts
│   │   ├── useAuth.ts
│   │   └── useSelection.ts
│   │
│   ├── stores/              # Zustandストア
│   │   ├── authStore.ts
│   │   └── selectionStore.ts
│   │
│   ├── types/               # 型定義
│   │   ├── image.ts
│   │   ├── tag.ts
│   │   └── search.ts
│   │
│   └── utils/               # ユーティリティ
│       ├── format.ts
│       ├── constants.ts
│       └── searchParams.ts  # 検索パラメータ変換
│
├── public/
├── index.html
├── tailwind.config.js
├── tsconfig.json
├── vite.config.ts
├── Dockerfile
└── package.json
```

---

## データフロー

### 画像取り込みフロー

```text
1. ユーザーが画像を /import フォルダに配置
              ↓
2. watchdog がファイル作成を検知
              ↓
3. SHA-256ハッシュを計算、重複チェック
              ↓
4. 重複あり → スキップ（ログ出力）
   重複なし → 続行
              ↓
5. PNGメタデータ（tEXtチャンク）を読み取り
              ↓
6. ParserFactoryでパーサーを自動選択
              ↓
7. メタデータをパース、共通フィールド抽出
              ↓
8. UUID v7を生成
              ↓
9. サムネイル生成（300x300 WebP）
              ↓
10. ファイルをストレージに移動
              ↓
11. DBにレコード挿入
              ↓
12. 元ファイルを削除（または処理済みフォルダへ移動）
```

### 検索フロー

```text
1. ユーザーが検索条件を入力
              ↓
2. フロントエンドがクエリパラメータを構築
              ↓
3. GET /api/images?{params} を呼び出し
              ↓
4. バックエンドでクエリを構築
   - カラム検索 → 通常のWHERE句
   - 全文検索 → tsvector使用
   - JSONB検索 → @>, ?演算子使用
              ↓
5. ページネーション適用（LIMIT/OFFSET）
              ↓
6. 結果をJSON返却
              ↓
7. フロントエンドで描画
```

---

## セキュリティ考慮事項

| 項目 | 対策 |
|------|------|
| 認証 | セッションCookie（HttpOnly, Secure, SameSite=Strict） |
| CSRF | SameSite Cookie + Originヘッダー検証 |
| SQLインジェクション | SQLAlchemy ORMによるパラメータバインド |
| パストラバーサル | ファイルパス正規化、ホワイトリスト検証 |
| XSS | React自動エスケープ、CSP設定 |

---

## パフォーマンス考慮事項

| 項目 | 対策 |
|------|------|
| DB接続 | コネクションプーリング（asyncpg） |
| 画像配信 | 静的ファイルとしてNginxから直接配信 |
| サムネイル | WebP形式で容量削減 |
| 一覧取得 | インデックス活用、必要カラムのみSELECT |
| フロントエンド | TanStack Queryによるキャッシュ |
