# Rust (axum) バックエンド再構築

FastAPI バックエンドを Rust + axum へ移植する **strangler-fig** リライトの設計・現状・移行手順をまとめる。コードは `backend-rs/` 配下。

## 目的と方針

- **AIメタデータ管理に特化**した高速・低メモリのバックエンドへ刷新する。
- **段階移行（strangler-fig）**: 既存の FastAPI バックエンドと同じ DB・`import/`・`storage/` を共有し、別ポート（既定 `8001`）で並走させる。エンドポイント単位で前段（nginx 等）の振り分けを切り替え、最終的に Rust 版へ寄せる。
- **設定・認証の互換**: `backend-rs` の `Config` は Python の `Settings` をフィールド単位でミラーしており、**同じ `.env` がそのまま使える**。JWT は同一 `SECRET_KEY`・HS256 のため、Python が発行したセッション Cookie は Rust 側でもそのまま有効（カットオーバー時に再ログイン不要）。
- **DB はスキーマ非破壊**: 既存の `comfyui_gallery`（`images` / `showcases` / `showcase_images` / `search_presets` / `smart_folders`）をそのまま読む。~~`backend-rs` はマイグレーションを持たず、スキーマ管理は引き続き Python(Alembic) / `db/init` 側が担う。~~ → **2026-07-25 更新: スキーマ管理は `backend-rs/migrations/` に一本化した**（Python backend と `db/init/` は廃止。docs/13 の B1、手順は `docs/runbooks/schema-baseline-migration.md`）。

## アーキテクチャ

レイヤリングは「HTTP ハンドラ → ストア（データアクセス）→ DTO（API 契約）」で統一。

```
backend-rs/src/
├── main.rs            エントリポイント（設定読込・DBプール・ワーカー起動・serve）
├── config.rs          環境変数設定（Python Settings 互換）
├── db.rs              PgPool 生成（sqlx）
├── error.rs           AppError → FastAPI 互換 {"detail": ...} エンベロープ
├── auth/              パスワード検証・JWT セッション（HS256, python-jose 互換）
├── http/              axum ハンドラ + ルーター + CORS + /storage 静的配信
├── dto/               リクエスト/レスポンス型（serde）。Falcon 互換の superset
├── image/             images テーブルのストア（検索・CRUD・近傍ナビ）
├── preset/ smart_folder/ showcase/   各テーブルの CRUD ストア
├── catalog/           model_name / loras JSONB の集計（models/loras API）
├── stats/             統計集計（FILTER / CASE / jsonb_array_elements）
├── tag/ batch/ duplicate/ export/    タグ一覧・一括操作・重複・エクスポート
├── parser/            メタデータパーサー（ComfyUI / A1111 / Forge / NovelAI）
├── media/             SHA-256 / 寸法 / WebP サムネイル / PNG・JPEG メタ読取
├── worker/            取り込みワーカー（notify 監視 + 定期スキャン）
├── civitai/ gelbooru/ 外部 API クライアント（reqwest + rustls）
└── util.rs            共通ヘルパー（escape_like / round2 / HTTP client）
```

### Falcon 互換（DTO superset）

`dto/image.rs` のレスポンスは、現行 React フロントと Falcon 連携の **両方を同時に満たす superset**:

- React: `storage_path` / `thumbnail_path` とフラットなページネーション（`total/page/per_page/total_pages`）。
- Falcon: `original_url` / `thumbnail_url`、ネストした `pagination`（`total_items/has_next/...`）、リスト項目の `user_tags` / `needs_improvement`。

両方を出力することで、フロントを壊さずに Falcon 連携を修復する。

## 移植済み範囲

### API（全 14 系統 = Python と同等）

| 系統 | エンドポイント | 備考 |
|---|---|---|
| health | `/` `/health` `/health/db` | |
| auth | `/api/auth/{login,logout,me}` | Cookie セッション |
| images | `GET/PATCH/DELETE /api/images/{id}`, `GET /api/images`, `POST /api/images/{id}/restore` | 全フィルタ（`q`/`orientation`/`seed±tol`/`date_from`/`xyz`/`upscaled`/`min_*`/`showcase_id`）・prev/next・showcase 内ナビ |
| bulk | `/api/bulk/{update,delete,restore}` | 一括（タグ集合演算はトランザクション内） |
| export | `/api/export/{metadata,prompts}` | JSON / CSV(CRLF) / プロンプトtxt |
| duplicates | `GET/DELETE /api/duplicates`, `DELETE /api/duplicates/{filename}` | `import/duplicated/` のFS操作・パストラバーサル対策 |
| search-presets | `/api/search-presets` CRUD | `SearchFilters` を型付け保存（exclude_none 相当） |
| smart-folders | `/api/smart-folders` CRUD | |
| showcases | `/api/showcases` …（9 ルート） | コレクション・画像追加/削除/並べ替え・カバー画像 |
| stats | `/api/stats` ほか 6 ルート | overview/分布/時系列/rating-analysis/model-rating-distribution |
| models | `/api/models`, `…/detail`, `…/civitai` | ベース名グルーピング（バージョンサフィックス正規化） |
| loras | `/api/loras`, `…/detail`, `…/civitai` | hash 優先の CivitAI 照会 |
| gelbooru | `/api/gelbooru/tags` | 429/503/502 を伝搬 |

### 取り込みパイプライン

- **パーサー**: ComfyUI（ノードグラフ走査・参照解決、再帰深さ制限あり）、A1111/Forge（parameters 文字列、`<lora:...>` 抽出）、NovelAI（`Comment` JSON）。`serde_json` は `preserve_order` を有効化し、ノード走査順を Python(挿入順) と一致させている。
- **メディア**: SHA-256（ストリーミング）、寸法取得、**ロッシー WebP サムネイル（品質85, libwebp）**、PNG tEXt/iTXt/zTXt と JPEG EXIF UserComment の読取。
- **ワーカー**: `notify` によるフォルダ監視 + 30 秒間隔の定期スキャン。ハッシュ重複は `import/duplicated/` へ退避。ストレージ命名は Python と完全一致（`ab/cd/<hash>.<ext>`, サムネは `thumbnails/ab/cd/<hash>.webp`）。

### テスト

`cargo test` で auth（JWT）・パーサー（ComfyUI/A1111）・ストレージパス・**ルーター構築（ルート衝突検出）** を検証。DB を必要としない単体テストのみ。

## 実行方法

開発（既存 compose に同居、同じ DB を共有）:

```bash
# .env は既存のものをそのまま利用（BACKEND_RS_PORT / RS_WATCHER_ENABLED を追加可）
docker compose up -d --build backend-rs
# → http://localhost:8001/health 、API は http://localhost:8001/api/...
```

- `BACKEND_RS_PORT`（既定 `8001`）: FastAPI(8000) と並走するためのポート。
- `RS_WATCHER_ENABLED`（既定 `false`）: Rust 側の取り込みワーカー。移行中は Python が取り込みを担当するため **false** 推奨。Rust に取り込みを移す段で `true` に。

本番イメージのビルド＆プッシュは `scripts/deploy.sh`（`urusy7/promptbox-backend-rs` を追加済み）。

## strangler-fig 移行手順（推奨）

1. `backend-rs` を 8001 で並走（`RS_WATCHER_ENABLED=false`）。読取系 API のレスポンスを Python 版とゴールデン比較で突き合わせる。
2. nginx 等で **読取系ルートから順に** `backend-rs` へ振り替え（images 一覧/詳細 → stats → models/loras …）。
3. 書込系（PATCH/DELETE/bulk/showcases/presets）を切り替え。
4. 取り込みを Rust へ移管: Python のワーカーを停止し、`RS_WATCHER_ENABLED=true`。
5. Python バックエンドを撤去。

各段は逆順で即ロールバック可能（同一 DB・同一セッション鍵のため状態は共有）。

## 残作業 / 今後

- **フロントエンドの Svelte 再構築**: 別アプリ規模のため本リライトのスコープ外。現状は既存 React(`frontend/`)が Rust/Python どちらのバックエンドとも通信可能（API 契約互換）。Svelte 化は独立タスクとして起票推奨。
- **キャッシュ**: Python が持つインメモリ TTL キャッシュ（tags 5分 / stats 10分 / CivitAI 24h / Gelbooru 5分）は未移植。結果は常に最新で正しいが、CivitAI/Gelbooru のレート制限対策として後日キャッシュ層を追加するとよい。
- **CSV エクスポートの微差**: 真偽値は Python の `str(bool)` に合わせ `True/False`。浮動小数の文字列表現は Rust 既定（`7` vs `7.0` 等の軽微差）。
- **物理削除とファイル**: 画像の物理削除は DB 行のみ削除（Python と同一挙動）。ディスク上の孤児ファイル回収は別途メンテ作業。
- **統合テスト**: DB を使う統合テスト（テスト用 Postgres コンテナ）は未整備。
