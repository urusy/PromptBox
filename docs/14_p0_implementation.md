# P0 実装計画（docs/13 ロードマップの「すぐ（P0）」）

作成日: 2026-07-25 / ステータス: **実装完了（全9項目・未コミット・本番未デプロイ）**

## 完了サマリ

| | 結果 |
|---|---|
| 実装項目 | B1 / B2 / C1 / B14 / A3 / A2(a) / B11 / B5 / A1 の**9項目すべて** |
| テスト | 25本 → **122本**（`cargo test`、2回連続で安定） |
| 静的解析 | `cargo clippy --all-targets` 警告0 |
| ビルド | `cargo build --release` 成功 |
| マイグレーション | 5本（initial_schema / baseline / fulltext_simple / jobs / image_events）。ローカル DB 適用済み |
| 新規 API | `/api/version` `/api/config` `/api/_manifest` `/api/jobs`(4) `/api/changes` |

**残作業**: コミット（規約により指示待ち）、本番デプロイ（`docs/runbooks/schema-baseline-migration.md` の
手順が**必須**）、Falcon 側の契約テスト実装。

`docs/13_backend_roadmap.md` §5 の P0 9項目を上から順に実装する。本ドキュメントは**進捗の書き戻し先**であり、
1フェーズ完了ごとに「済」と次の一手を追記する（中断・モデル切替に備える）。

## 着手前の決定（docs/13 §6）

| 決定事項 | 結論 | 影響 |
|---|---|---|
| 実装範囲 | **P0 9項目を上から順に全部** | 下表のフェーズ1〜9 |
| 評価（rating / user_tags / memo）のマスタ | **PromptBox** | A1 変更フィードは INSERT/UPDATE/DELETE を全種発行。将来 A7（同期状態）で Falcon へ一方向同期 |
| showcases / presets / smart_folders の保存先 | 未決（P1 の C5 着手時に再判断） | 本 P0 では触らない（凍結扱い） |

## 事前調査で確定した事実（2026-07-25 観測）

- ローカル DB（`promptbox-db-1` / `comfyui_gallery`）の `images` 実スキーマは `db/init/01_init.sql` と**完全一致**。
  `alembic_version` は `0001_baseline` のみ＝alembic による差分適用は実質ゼロ。
  → `db/init/{01,02,03}.sql` をそのまま初期マイグレーションに昇格して安全。
- `_sqlx_migrations` には `20260711000001 baseline` の1行のみ。
- ローカル DB は `images` 0件（開発用）。本番は約41,680件。
- ローカル compose の `db` はポート未公開 → ホストから `cargo test` で繋げない（フェーズ2で対応）。

## フェーズ一覧

| # | 項目 | 内容 | 状態 |
|---|---|---|---|
| 1 | **B1** スキーマの単一の真実 | `db/init/*` → `migrations/20260711000000_initial_schema.sql` に昇格。既存 DB には `_sqlx_migrations` へ手動 INSERT（checksum は sha384）。`db/init/` 削除 + compose のマウント除去 | **済**（2026-07-25） |
| 2 | **B2** 統合テスト基盤 | `#[sqlx::test(migrations = "./migrations")]` で `backend-rs/tests/` を新設。優先: `push_filters` 全分岐 / `neighbors` 4方向 / bulk 500件上限 / `include_deleted` の意味 | **済**（2026-07-25） |
| 3 | **C1** 全文検索の作り直し | `simple` + `websearch_to_tsquery` + 生成列 `search_vector` + GIN、trigram フォールバック。不正クエリの 500 が消える | **済**（2026-07-25） |
| 4 | **B14** `/api/version`・`/api/config` | version / git_sha / schema_version / parser_version / features / limits を返す | **済**（2026-07-25） |
| 5 | **A3** `warnings[]` + `?strict=true` | 未知パラメータ・clamp・フォールバックを非破壊で通知。`X-PromptBox-Warnings` ヘッダ併用 | **済**（2026-07-25） |
| 6 | **A2(a)** ルートマニフェスト | `GET /api/_manifest` をルータ定義から生成。Falcon の `proxy_routes.go` 契約テストの対向 | **済**（2026-07-25） |
| 7 | **B11** ミドルウェア一式 | ボディサイズ / タイムアウト（`/storage/` 除外）/ 並行数 / レート制限 | **済**（2026-07-25） |
| 8 | **B5** 軽量ジョブ基盤 | `jobs` テーブル + `POST /api/jobs` / `GET /api/jobs/{id}` / cancel。Tokio + `Semaphore(1)`、起動時に `running`→`interrupted` 回収 | **済**（2026-07-25） |
| 9 | **A1** 変更フィード | `image_events` を DB トリガで記録し `GET /api/changes?since=<seq>`。トゥームストーン込み | **済**（2026-07-25） |

## 検証ゲート（各フェーズ共通）

```bash
cd backend-rs
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test            # フェーズ2以降は統合テスト込み（要 DATABASE_URL）
cargo build --release # 最終フェーズのみ
```

## 本番適用時の注意（フェーズ1）

**⚠️ `_sqlx_migrations` への手動 INSERT を誤ると、本番で `CREATE TABLE images` が再実行されて起動不能になる。**
手順とロールバックは `docs/runbooks/schema-baseline-migration.md`（フェーズ1で作成）に記載する。
本番 DB への適用はユーザーが実行する。

## 既知の技術的負債（本 P0 では触らない）

- **`cargo fmt --check` がリポジトリ全体で失敗する**（約70箇所）。edition 2024 の rustfmt スタイル差分で、
  ほぼ全ファイルの `use` 周辺が対象。P0 の変更とは無関係だが、一括適用すると巨大な差分になるため
  独立したコミットで別途実施する。**検証ゲートは当面 clippy + test を主軸にする。**
- `alembic_version` テーブルが既存 DB に残っている（Python 時代の遺物、実害なし）。

## 進捗ログ

- 2026-07-25: 事前調査完了（上記「事前調査で確定した事実」）。フェーズ1に着手。
- 2026-07-25: **フェーズ1（B1）完了**。
  - 新規 `backend-rs/migrations/20260711000000_initial_schema.sql`（旧 `db/init/{01,02,03}.sql` の統合）
  - `db/init/` と `db/migrations/`（Python 時代の手動マイグレーション）を削除
  - `docker-compose.yml` / `docker-compose.prod.yml` の `docker-entrypoint-initdb.d` マウントを除去
  - `docker-compose.yml` の db に `${DB_PORT:-5433}:5432` を公開（フェーズ2 の統合テスト用）、`.env.example` に追記
  - 連動修正: `main.rs` / `smart_folder/mod.rs` のコメント、docs/03・06・11 の記述
  - 新規 `docs/runbooks/schema-baseline-migration.md`（本番の `_sqlx_migrations` 手動 INSERT 手順）
  - **検証**: 空 DB への `sqlx migrate run` 成功 → `pg_dump --schema-only` が実 DB と完全一致（差分は
    `_sqlx_migrations` と `alembic_version` のみ）／既存 DB では手動 INSERT 後 `migrate run` が no-op ／
    `cargo clippy --all-targets` 警告0 ／ `cargo test` 25 passed
  - **学び**: 適用済みマイグレーションはコメント1行でも編集不可（sha384 検証で VersionMismatch）。
    実際に baseline.sql の文言を直しかけて気づき、復元した。
- 2026-07-25: **フェーズ2（B2）完了**。テスト 25本 → **61本**。
  - **lib クレート化**: 新規 `src/lib.rs`（全モジュールを `pub mod`）、`main.rs` は `promptbox::` を使う薄い
    エントリポイントに。バイナリ専用モジュールは統合テストから参照できないため必須の変更。
  - 新規 `tests/common/mod.rs`（`NewImage` フィクスチャ + showcase/tag ヘルパー）
  - 新規 `tests/image_search.rs`（17本）— `include_deleted` が「削除済みのみ」であること、rating の優先順位、
    LIKE エスケープ、tags の AND、JSONB containment、model_params フラグの NULL 扱い、orientation、
    date_from の許容形式、seed トレランス（i64 飽和込み）、sort whitelist のフォールバック、
    count と page の一致
  - 新規 `tests/image_neighbors.rs`（8本）— prev/next 4方向、端の None、フィルタ・論理削除のスキップ、
    showcase の curated order、非メンバー
  - 新規 `tests/batch_ops.rs`（10本）— 削除済みを更新しない、空更新の no-op、タグ merge（重複なし・
    remove→add の順）、soft_delete / restore / delete_permanent の対象、未知 id の無視
  - `src/http/bulk.rs` に `validate_ids` の境界テスト（0 / 1 / 500 / 501）
  - `image::SearchParams` に `Default` を実装（テストの可読性 + 既定値の一元化）
  - `docker-compose.yml` の db 公開ポートを利用し、`CLAUDE.md` のテスト方針に実行手順を追記
  - **検証**: `cargo test` **61 passed** ／ `cargo clippy --all-targets` 警告0
- 2026-07-25: **フェーズ3（C1）完了**。テスト 61本 → **70本**。
  - 新規 `migrations/20260725000001_fulltext_simple.sql`
    — `images.search_vector`（`to_tsvector('simple', coalesce(positive_prompt,''))` の STORED 生成列）+ GIN、
    `positive_prompt` の `gin_trgm_ops` index を追加、未使用だった english FTS index 2本を削除
  - `image::push_filters` の `q` 分岐を
    `search_vector @@ websearch_to_tsquery('simple', q) OR positive_prompt ILIKE %q%` に変更（両armとも index-backed）
  - `tests/image_search.rs` に9本追加 — 大小文字無視 / 複数語 AND / **`!` `|` `(` `:` でエラーにならない**
    （旧実装は 500）/ `(masterpiece:1.2)` の literal 一致 / 空白区切り日本語 / **区切りなし日本語の部分一致**
    （`少女` → `美少女イラスト`、trigram arm でしか通らない）/ `"引用フレーズ"` と `-除外` / 空白のみのクエリは無視 /
    prompt が NULL の画像は不一致
  - `docs/runbooks/schema-baseline-migration.md` に「以降のマイグレーション」節を追記
    （生成列追加は ACCESS EXCLUSIVE でテーブル書き換え、約42,000行で数秒〜十数秒）
  - **検証**: `cargo test` **70 passed** ／ `cargo clippy --all-targets` 警告0 ／
    既存ローカル DB への `sqlx migrate run` 成功（生成列と index を実DBで確認）
- 2026-07-25: **フェーズ4（B14）完了**。テスト 70本 → **75本**。
  - 新規 `build.rs` — `GIT_SHA`（Docker では build arg、ローカルでは `git rev-parse`、無ければ `unknown`）と
    ビルド時刻（epoch 秒、`SOURCE_DATE_EPOCH` 尊重）を `env!` で埋め込む
  - 新規 `src/dto/meta.rs` / `src/http/meta.rs` — `GET /api/version`（**無認証**: 接続前の互換性確認用。
    version / git_sha / built_at / schema_version（`max(_sqlx_migrations.version)`、DB 不通なら null）/ parser_version）と
    `GET /api/config`（**要認証**: features / limits / storage_backend / thumbnail_sizes。**秘密は一切含めない**）
  - `parser::VERSION = 1` を新設（B3/B4 の再解析で使う布石）
  - `image::ALLOWED_SORT_COLUMNS` を公開（A3 と共有する契約に）
  - Dockerfile に `ARG GIT_SHA`、`docker-compose.yml` に build args、
    `scripts/docker-build-push.sh` は backend-rs ビルド時に自動で `--build-arg GIT_SHA=$(git rev-parse --short HEAD)`
  - `Config::for_test()` を `#[cfg(test)]` から外した（統合テストは別クレートのため）
  - `tests/common/mod.rs` に HTTP テスト基盤（`test_router` / `session_cookie` / `get_json`、dev-dep に tower util）
  - 新規 `tests/meta_api.rs`（5本）— version が無認証で 200 / schema_version が実適用値と一致 /
    config が未認証で 401 / limits と features / **config に秘密が混ざらない**
- 2026-07-25: **フェーズ5（A3）完了**。テスト 75本 → **89本**。
  - 新規 `src/http/warnings.rs` — `Warning{code, param, message, hint}` と収集器。
    `code` は `unknown_param` / `clamped` / `fallback`。`hint` は編集距離2以内 **または前方一致**で候補を提案
    （`sampler` → `sampler_name` は距離7だが前方一致で拾える＝Plan 222 の事故そのもの）
  - `GET /api/images` に適用 — `RawQuery` で未知パラメータを検出、`page`/`per_page` の clamp、
    `sort_by` のフォールバックを通知。`ImageListResponse.warnings` は**空なら丸ごと省略**（既存クライアント無影響）＋
    `X-PromptBox-Warnings` ヘッダ。`?strict=true` で 400
  - `ListQuery::KNOWN_PARAMS` を定義（構造体との同期が必要な旨をコメントで明記）
  - ユニットテスト7本（`unknown_params` の解析・percent decode・hint・Levenshtein）＋
    新規 `tests/warnings_api.rs`（7本）— **正常リクエストは warnings キーもヘッダも出ない** /
    `sampler` に hint 付き警告 / clamp / sort フォールバック / Falcon の `sort=` エイリアスは警告しない /
    strict で 400・正しい要求は通る / 複数警告の累積
  - **検証**: `cargo test` **89 passed** ／ `cargo clippy --all-targets` 警告0
- 2026-07-25: **フェーズ6（A2(a)）完了**。テスト 89本 → **97本**。全44ルートを宣言。
  - 新規 `src/http/manifest.rs` — `RouteSpec{path, methods, auth, query}` の表と `GET /api/_manifest`（無認証）
  - **乖離を検出する仕組み**（axum は構築済み Router を列挙できないため）:
    - ユニットテスト: `include_str!("mod.rs")` から `.route("…")` を正規表現で抽出し、マニフェストのパス集合
      （`/api` プレフィックスを除去）と**完全一致**を要求 → 「ルータにあるが未宣言」を検出
    - 契約テスト `tests/manifest_api.rs`（5本）: 宣言した全パスに **TRACE** を投げて 405 を確認
      （パス未登録なら 404 になるので、両者を区別できる）→ 「宣言したが存在しない」を検出。
      さらに `auth: true` の**全メソッド**が匿名で 401 / `auth: false` は 401 にならない /
      有効セッションなら 401 にならない、を検証
  - Falcon 側の対向（別リポジトリ・未実施）: CI で `GET /api/_manifest` を取得し、`proxy_routes.go` の
    allow-list と突合するテストを1本置く
  - **検証**: `cargo test` **97 passed** ／ `cargo clippy --all-targets` 警告0
- 2026-07-25: **フェーズ7（B11）完了**。テスト 97本 → **102本**。
  - ルータを2スタックに分割 — JSON 系には `RequestBodyLimitLayer(1MB)` /
    `TimeoutLayer::with_status_code(408, 30s)` / `GlobalConcurrencyLimitLayer(64)`、
    **`/storage/` は3つとも適用外**（10MB級の原本ストリーミングがタイムアウトで切れる・画像大量表示で枠を食う）
  - `/api/auth/login` のみ `tower_governor`（burst 10・2秒に1補充、`SmartIpKeyExtractor`）。
    依存は `--no-default-features --features axum` で追加（既定だと tonic まで引き込むため）
  - `main.rs` を `into_make_service_with_connect_info::<SocketAddr>()` に変更
    （プロキシヘッダが無い直アクセスでもクライアント識別ができるように）
  - `GlobalConcurrencyLimitLayer` を選択（`ConcurrencyLimitLayer` は clone ごとに別セマフォになり、
    axum のように Service を clone する構成では効かない）
  - 新規 `tests/middleware.rs`（5本）— 1MB 超で 413 / 500件の bulk は通る /
    login 20連打で 429 が出る / **レート制限はクライアント別**（他IPは通常応答）/ `/storage/` は素通り
  - 既存テストの修正: `oneshot` は ConnectInfo を持たないため、レート制限のキー抽出用に
    `x-forwarded-for` を明示（本番は nginx のヘッダか ConnectInfo で必ず取れる）
  - ライブラリ API は ctx7 で確認（`TimeoutLayer::new` は deprecated → `with_status_code`）
  - **検証**: `cargo test` **102 passed** ／ `cargo clippy --all-targets` 警告0
- 2026-07-25: **フェーズ8（B5）完了**。テスト 102本 → **110本**。
  - 新規 `migrations/20260725000002_jobs.sql`（`jobs` テーブル）、`src/job/mod.rs`、`src/http/jobs.rs`
  - `POST /api/jobs`（202 + queued 行）/ `GET /api/jobs`（status フィルタ）/ `GET /api/jobs/{id}` /
    `POST /api/jobs/{id}/cancel`。`AppState.jobs` にレジストリを保持
  - 実行は Tokio タスク + `Semaphore(1)`、キャンセルは協調的（フラグをジョブが確認）、
    **起動時に queued/running を `interrupted` へ回収**（`main.rs`）
  - 動作確認用に `noop` kind（steps / sleep_ms）を用意。未知 kind は 400 で拒否
  - **テストが実バグを検出**: キャンセルフラグを `run()` 内で登録していたため、
    permit 待ちの間に届いたキャンセルを取りこぼしていた → 登録を `enqueue()`（spawn 前）に移動し、
    claim 直後にもフラグを確認する二段構えに修正
  - 新規 `tests/jobs_api.rs`（8本）— 202 と完走 / 未知 kind の 400 / **実行中の進捗が見える** /
    実行中のキャンセル / status フィルタ / 404 / **起動時スイープ（終了済みは触らない）** /
    **同時実行が1件を超えない**
- 2026-07-25: **フェーズ9（A1）完了**。テスト 110本 → **122本**。P0 完了。
  - 新規 `migrations/20260725000003_image_events.sql` — `image_events(seq BIGSERIAL, image_id, kind,
    occurred_at, payload)` と `images` への AFTER INSERT/UPDATE/DELETE トリガー。
    **アプリではなく DB 側に置いた**ので、取り込みワーカー・一括操作・PATCH・将来のジョブが自動的に発行する
  - kind は `created` / `updated` / `deleted`（論理）/ `restored` / `purged`（物理・トゥームストーン）。
    `updated` の payload には**変更列の一覧**（rating / is_favorite / needs_improvement / user_tags / user_memo、
    それ以外の変化は `metadata`）。**実質変化のない UPDATE はイベントを出さない**
  - 既存ライブラリ向けに **created イベントを backfill**（since=0 から全件を取得できる）
  - 新規 `src/change/mod.rs` / `src/http/changes.rs` — `GET /api/changes?since=&limit=&compact=`。
    `{events, next_since, has_more, latest_seq}`。compact は画像ごとに最新1件へ圧縮（サブクエリで圧縮してから
    seq 順に切る＝カーソルが壊れない）
  - `ImageUpdate` に `Default` を実装
  - 新規 `tests/changes_api.rs`（12本）— created / rating 変更の changed / タグ・メモ / **no-op は無イベント** /
    メタデータのみの変更 / delete・restore・purge の区別とトゥームストーンの中身 / **一括操作でも発行される** /
    カーソルの連続性（重複も欠落もなし・空ページでカーソルが戻らない）/ compact / backfill / 要認証 /
    トゥームストーンが行より長生きする
  - **検証**: `cargo test` **122 passed**（2回連続で安定）／ `cargo clippy --all-targets` 警告0 ／
    `cargo build --release` 成功 ／ ローカル開発 DB に全5マイグレーション適用済み
