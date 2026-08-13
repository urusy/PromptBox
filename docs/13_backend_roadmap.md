# バックエンド機能ロードマップ（headless エンジン化を前提としたブレスト）

作成日: 2026-07-25 / ステータス: ブレスト（着手判断待ち）

`backend-rs` が今後どんな機能を持つべきかを、現状のコードを実際に読んだうえで洗い出したもの。**本ドキュメントの作成時点でコードは一切変更していない。**

## 背景 — なぜ今これを考えるのか

`docs/12_falcon_integration.md` の推奨は **案C フェーズ2（C-2）**＝「Svelte フロントは作らず、閲覧・整理 UI を Falcon に寄せ、**PromptBox は worker + parser + API だけを headless 運用**する」。

そして Falcon 側の **Plan 224（2026-07-25, commit `d7d221f`）で PromptBox のフロント機能 11 ルートが全移植された**（一覧 / 詳細 / 編集 / 一括 / ゴミ箱 / Showcase / スマートフォルダ / 統計 / カタログ / 重複 / Gelbooru タグ / クイック評価）。

→ **C-2 は意思決定を待たずに事実上実行済み**。よって「PromptBox のバックエンドが持つべき機能」は、**UI を持つアプリの機能追加**ではなく、**取り込み・メタデータエンジン兼 API サーバとしての責務の深化**として設計されるべき。本ドキュメントはその前提で書く。

検討の観点は4つすべて（①Falcon 連携 ②基盤・運用 ③検索・発見 ④生成ナレッジ）。**マルチユーザー化はしない**（シングルユーザー維持。機械アクセスの認可整備は可）。

---

## 1. 調査で確定した事実（観測ベース）

すべてコードを読んで確認済み。推測を含む箇所は「※推測」と明記する。

### 現状のスタック

Rust 2024 / axum 0.8 / sqlx 0.8（生 SQL + QueryBuilder、ORM 無し）/ PostgreSQL 16 / object_store → MinIO(S3) or fs / notify によるフォルダ監視ワーカー / JWT(HS256)+bcrypt のシングルユーザー（`users` テーブル無し）。API 約40本。

### 検討前の認識を修正すべきだった2点

| # | 当初の認識 | 実際 | 出典 |
|---|---|---|---|
| ① | 「migration 運用が無い」 | **`sqlx::migrate!("./migrations").run()` は起動時に実行されている**。新ファイルを置けば本番に当たる。壊れているのは「`migrations/` だけで空 DB を再構成できない」点のみ → 効く先は**本番ではなく CI・統合テスト・DR リストア** | `backend-rs/src/main.rs:44`、`migrations/20260711000001_baseline.sql`（中身は `SELECT 1`） |
| ② | 「`raw_metadata` は ComfyUI のグラフ置き場」 | **全パーサが再パース可能な原データを保存している**。ComfyUI=`{prompt, workflow}` の両グラフ、A1111=`{parameters:"..."}`（入力文字列そのもの）、NovelAI=Comment JSON 全体 → **画像ファイルを読まずに全件を遡及再解析できる** | `src/parser/comfyui.rs:49-52`、`src/parser/a1111.rs:32`、`src/parser/novelai.rs:26` |

②は本検討で最も価値のある発見。**パーサを直したときに過去データを救える**ことを意味する（→ B3 / B4）。

### 未活用の資産

`raw_metadata` に ComfyUI のノードグラフ全体が入っているのに、読み出しは detail でそのまま返すだけ。`workflow_extras` は `node_count` と `workflow_version` の**2個しか抽出していない**（`src/parser/comfyui.rs:330-342`）。PromptBox にしかない資産がほぼ死蔵されている（→ D6）。

### 構造的な穴（確認済み）

| 領域 | 事実 | 出典 |
|---|---|---|
| テスト | **`backend-rs/tests/` が存在しない**。SQL を含む全ロジック（`push_filters` 全分岐 / catalog / stats / batch）が未検証。テストは parser・ルータ構築・storage path のみ | `ls tests/` → No such file |
| 全文検索 | index・クエリとも `to_tsvector('english', ...)`。**日本語プロンプトは実質検索不能**。`q` を空白→` & ` 置換して `to_tsquery` に直投入するため `!` `\|` `(` で構文エラー→**500** | `db/init/01_init.sql:67-68`、`src/image/mod.rs:213-225` |
| 類似検出 | 重複判定は **SHA-256 完全一致のみ**。pHash / 知覚ハッシュ無し（grep ヒット0） | `src/media/mod.rs:15-27` |
| スマートフォルダ | `filters` は**サーバで画像検索に適用されない**（CRUD のみ・112行）。件数も中身も返せない。`SearchFilters`(dto/preset.rs) ↔ `SearchParams`(image/mod.rs) の**変換関数が存在しない** | `src/smart_folder/mod.rs` |
| 取り込み | **アップロード API 無し**（`POST /api/images` 不在）。フォルダ監視のみ・**非再帰**・`IMPORT_PATH` は単数 | `src/worker/mod.rs:494` |
| 連携 | **Webhook / 変更フィード / `updated_since` カーソルが無い** → 外部は全件ポーリングしかできない | router 定義に該当なし |
| 可観測性 | `import/failed/` `skipped/` はブラックボックス（API 無し・tracing ログのみ）。`duplicated/` だけ API 化されていて不均衡 | `src/duplicate/mod.rs` |
| GC | 物理削除時の孤児オブジェクト回収なし（コメントで "orphan object is harmless"）。ゴミ箱の自動パージも無し | `src/storage/mod.rs:67-69` |
| ミドルウェア | レート制限・ボディサイズ制限・タイムアウトいずれも無し（bulk の ids 500件上限のみ） | `src/http/mod.rs` の router |
| 契約 | per_page は**黙って** 1〜120 に clamp、sort 不明値は**黙って** created_at にフォールバック、serde は未知フィールドを**黙って**破棄（実際に `sampler` vs `sampler_name` で無言 fail 事故） | `src/image/mod.rs`、Falcon `client.go` のコメント |
| 表現の不統一 | 画像 API は `thumbnail_url`（`/storage/` 付き）、Showcase API は `thumbnail_path`（DB 生値） | Falcon Plan 226 の真因 |
| 命名 | `include_deleted=true` の実際の意味は「**削除済みのみ**」 | `src/image/mod.rs:152-156` |
| 配信 | `/storage/{*path}` は**無認証**（意図的）。Range 未対応。サムネは長辺300の1サイズのみ | `src/http/storage.rs` |

### Falcon 側のコードが発しているシグナル（一次情報）

`Falcon/gateway/internal/infrastructure/promptbox/proxy_routes.go` のコメント:

> When PromptBox's router changes, update this table — the pass-through does not validate payloads, so **a silent drift would only surface as a 404 in the UI**.

`client.go`（Plan 222 の事故を受けた注記）:

> PromptBox の ListQuery は sampler_name で受ける（sampler では**黙って捨てられる**）

→ Falcon は PromptBox のルータを**手書きの正規表現でミラー**しており、ドリフトは 404 でしか気づけない。しかも無言破棄で一度事故済み。**「あったら便利な API」より、契約の機械化が先**という強い示唆。

---

## 2. 責務の線引き（アイデアの前に決める背骨）

C-2 が実行済みである以上、PromptBox は「機能を足すアプリ」ではなく「**責務を絞って深くするエンジン**」であるべき。

| 領域 | 担当 | 理由 |
|---|---|---|
| メタデータ抽出・正規化・**再解析** | **PromptBox** | 唯一のパーサ保持者（doc12 §5：Falcon は AI メタパーサを持たない） |
| コンテンツ同一性（SHA-256 / pHash / prompt_hash / workflow_hash） | **PromptBox** | 取り込み時にしか安く計算できない |
| 生成ナレッジ集計（モデル / LoRA / タグ × 評価） | **PromptBox** | AI メタを持つスキーマは PromptBox 側だけ |
| CivitAI / Gelbooru 連携 | **PromptBox** | 既存資産 |
| 取り込みの信頼性（監視・隔離・再投入・進捗） | **PromptBox** | ワーカーの所有者 |
| **変更フィードの発行** | **PromptBox** | 上流が発行しないと下流は同期できない |
| 閲覧・整理 UI 一切 | Falcon | Plan 224 で移植済み |
| タグの正規化・階層・グループ | Falcon | Falcon は正規化テーブル、PromptBox は JSONB 配列（劣位） |
| コレクション（= showcases） | Falcon | `collections` が上位互換 |
| 長期保存・冗長化・動画・配信 | Falcon | DAM 本来の仕事 |
| スマートフォルダ / プリセットの**保存** | Falcon | ただし **filters の評価エンジンは PromptBox**（C5） |

**通奏低音: PromptBox は状態を持つのをやめ、「導出」と「契約」に集中する。**

---

## 3. アイデアカタログ

### 観点① Falcon 連携の強化

| ID | 案 | 何を解決するか | 実装の要点 | 工数 | 依存 |
|---|---|---|---|---|---|
| **A1** | **変更フィード + トゥームストーン** `GET /api/changes?since=<seq>` | doc12 §8 の宿題「取込後に PromptBox で評価を変えても再同期されない」。現状 Falcon は `source_url` 重複でスキップ＝**永久に一度きり**。削除も伝播しない | `image_events(seq BIGSERIAL, image_id, kind, occurred_at, payload)` を **DB トリガ**で書く（`images` の AFTER INSERT/UPDATE/DELETE）。API は `{events:[], next_since, has_more}`。同一 image_id は最新 seq のみ返す圧縮モードを用意 | M | B1 |
| **A2** | **ルートマニフェスト / 契約テスト** `GET /api/_manifest` | 上記の「404 でしか気づけない」の恒久対策。Falcon の `proxy_routes.go` を手で保守しなくてよくする | (a) 軽量: ルータ定義配列から `{routes:[{path, methods, query}]}` を生成 → Falcon 側 CI で allow-list と突合するテスト1本。(b) 本格: `utoipa`/`aide` で OpenAPI 3.1 | (a)S /(b)M | — |
| **A3** | **「黙って捨てない」プロトコル** `warnings[]` + `?strict=true` | 無言 clamp / 無言フォールバック / 未知フィールド破棄。Plan 222 の事故の再発防止。`deny_unknown_fields` は既存クライアントを壊すので使えない | `#[serde(flatten)] extra: HashMap` で未知キーを捕捉 → レスポンスに `warnings:[{code:"unknown_param", param:"sampler", hint:"did you mean sampler_name?"}]` を**非破壊で**付与。`X-PromptBox-Warnings` ヘッダも。`strict=true` なら 400。Falcon の CI/staging は strict で叩く | **S** | — |
| **A4** | **機械アクセス用 API キー**（マルチユーザー化なし） | Falcon が Cookie セッションを使い回している状態（失効不能・並行性・平文パスワードを Falcon 設定に保持） | `api_keys(id, name, key_hash, scopes[], last_used_at, expires_at, revoked_at)`。**users テーブルは作らない**（キー＝単一管理者の代理）。`CurrentUser` extractor を `Principal{Session\|ApiKey{scopes}}` に一般化。発行は CLI サブコマンド | M | B1 |
| **A5** | **`/storage/` の保護 + Range + 派生サイズ** | サムネ300の1サイズのみ＝詳細表示のたびに 10MB 級の原本が飛ぶ。Range 未対応 | `?w=800` でオンデマンドリサイズ + `derived/w800/` にキャッシュ put（既存画像の backfill 不要）。許可サイズは `{300,800,1600}` に固定列挙（DoS 対策）。Range は `object_store::get_opts` で薄く実装 | M | A4 |
| **A6** | **一括取得 + スパースフィールド** `POST /api/images/batch`, `?fields=` | Falcon の bulk-import が list → 各 id で detail の **N+1**。`ImageDetail` は `raw_metadata`（数十〜数百 KB）を常に含む | ids 上限500。`?exclude=raw_metadata` だけでも効果大。まず「全部作ってから JSON をフィルタ」（型を増やさない） | S〜M | — |
| **A7** | **同期状態の記録** `image_sync` | A1 の対。「どこまで吸ったか」を上流でも保持し `unsynced_to=falcon` で差分だけ返す | `image_sync(image_id, target, external_id, synced_seq, synced_at)` + `POST /api/sync/{target}/ack` | M | A1・**運用決定** |
| **A8** | **keyset ページング** `?after=` | offset のみ。doc12 §7 の「bulk-import 100ページ上限」は offset 前提の制約で cursor があれば消える | **既存 `neighbors()` が行値比較 `(col,id) < (subquery)` を実装済み**（`src/image/mod.rs:379-391`）→ **同じ手法をそのまま流用できる**。`?with_total=false` でカウントクエリも外す | S〜M | — |
| **A9** | **アップロード API** `POST /api/images` | 共有フォルダ必須の制約。他マシンの ComfyUI・iPhone・Falcon から直接投げられない | ワーカーの `import_image` と**同じ関数**を通す（現状 `&Path` 前提 → バイト列版へ切り出すリファクタが本体）。重複時 409 + 既存 id | M | B11 |
| **A10** | アウトバウンド Webhook（push） | 取り込み〜Falcon 反映のレイテンシ | `WEBHOOK_URL` + HMAC 署名、`webhook_deliveries` に積んで指数バックオフ | M | A1・B5 |

> **A10 は A1 の後回しでよい。** pull（A1）があれば必須ではなく、先に作ると再送・冪等の複雑さだけ背負う。

### 観点② 基盤・運用の健全化

| ID | 案 | 何を解決するか | 実装の要点 | 工数 | 依存 |
|---|---|---|---|---|---|
| **B1** | **スキーマの単一の真実**（`db/init` → `migrations` 一本化） | 「空 DB を migrations だけで作れない」→ **CI も統合テストも DR リストアも成立しない**。以下の機能はほぼ全部が新列 / 新テーブルを伴う | ① `db/init/{01,02,03}.sql` の内容を `migrations/20260711000000_initial_schema.sql` として登録（baseline より前の版番号）② 既存本番 DB には `_sqlx_migrations` に該当行を**手動 INSERT**（適用済みとマーク）③ `db/init/` を削除し compose の `docker-entrypoint-initdb.d` マウントを外す。**⚠️ ② を誤ると本番で `CREATE TABLE images` が再実行され起動不能。先に本番の `_sqlx_migrations` をダンプして確認する。`IF NOT EXISTS` で誤魔化すのは差分を隠すので悪手** | M | — |
| **B2** | **統合テスト基盤** `#[sqlx::test]` | `tests/` が存在しない。SQL を含む全ロジックが未検証 | sqlx 0.8 の `#[sqlx::test(migrations="./migrations")]` は**テストごとに独立 DB を自動作成・破棄**（testcontainers 不要、Postgres が1台あればよい）。優先: `push_filters` 全分岐 / `neighbors` 4方向 / bulk 500件上限 / `include_deleted` の意味 | M | B1 |
| **B3** | **パーサのリグレッション検知（再解析ドライラン）** | パーサを1行直したとき、本番全画像への影響を確認する手段が無い。ComfyUI のノードは頻繁に増えるので改修は不可避 | **§1② の発見が効く**: `raw_metadata` から画像ファイルを読まずに再パースできる。`POST /api/admin/reparse/dry-run` → `{changed, unchanged, diffs:[{image_id, field, before, after}], summary_by_field}`。実装のコアは **raw_metadata → png_info 相当の逆変換アダプタ**。CLI 版（`--dry-run --out diff.json`）も用意すれば CI に組める | M | B5 |
| **B4** | **再解析の実行 + `parser_version`** | **パーサを直しても過去データは腐ったまま**。新ノード対応前に取り込んだ数千枚は永久に `model_name=NULL` | `images.parser_version SMALLINT`, `parsed_at` を追加。`POST /api/admin/reparse {filter, preserve_user_fields:true}`。**rating / is_favorite / user_tags / user_memo は絶対に触らない**（UPDATE の SET 列をホワイトリスト定数化 + テスト必須）。`?parser_version_lt=5` で対象抽出 | M | B3・B5・B1 |
| **B5** | **軽量ジョブ基盤** ★レバレッジ最大 | 再解析・pHash backfill・派生サムネ・孤児 GC・ゴミ箱パージ・統計再集計・プロンプトトークン化・ワークフロー指紋が**全部同じ形**（長時間バックグラウンド＋進捗照会）。**個別に7回実装するか、1回作って7回使うか** | `jobs(id, kind, status, params, progress_current/total, result, error, timestamps)` + `POST /api/jobs`→202 / `GET /api/jobs/{id}` / cancel。実行は Tokio タスク + `Semaphore(1)`。**キューイング FW は入れない**（シングルインスタンス前提）。起動時に `running`→`interrupted` の残骸回収を必ず入れる | M | B1 |
| **B6** | **取り込みの可観測性** `GET /api/import/status` | failed / skipped がブラックボックス。**何がなぜ失敗したかを API から一切見られない**。Falcon に「取り込み状況」を出せない | `import_events(occurred_at, filename, outcome, reason, error_kind, image_id, duration_ms)`。`{watching, processing, last_scan_at, counts:{...}, recent_failures:[]}`。**`src/duplicate/mod.rs` の `info()` をディレクトリ名パラメータ化するだけで failed / skipped にも一般化できる** | S〜M | B1 |
| **B7** | **失敗ファイルの再投入 API** | B6 で見えても、直す手段が「NAS に SSH して mv」しかない。パーサ修正後の再投入が運用の主動作になるのに | `POST /api/import/retry {filenames}` / `rescan` / `DELETE /api/import/failed/{name}`。`is_safe_filename` ガードを流用。Worker ハンドルを `AppState` に持たせる小さなアーキ変更が要る | S | B6 |
| **B8** | 複数 / 再帰監視 + サブフォルダの意味づけ | `RecursiveMode::NonRecursive` かつ `IMPORT_PATH` 単数 → 日付サブフォルダ・複数マシン・ツール別フォルダに非対応 | `IMPORT_PATHS`(CSV) + Recursive。相対サブパスを `images.import_source` に記録し検索軸に。**⚠️ failed/ skipped/ duplicated/ は現在 NonRecursive で偶然除外されている** → 再帰化時に除外しないと無限ループ（quarantine → 再検出 → 再 quarantine）。除外パスの単体テスト必須 | M | B1 |
| **B10** | **孤児 GC + ゴミ箱自動パージ + 整合レポート** | 孤児オブジェクト回収なし。加えて**逆方向**（DB に行はあるが storage にオブジェクトが無い＝表示できない画像）が一切検知できない | `POST /api/jobs {kind:"gc", dry_run:true}` → storage list × DB の `storage_path ∪ thumbnail_path` を突合 → `{orphan_objects, missing_objects, reclaimable_bytes}`。**missing 側は絶対に自動削除しない**（レポートのみ）。パージは `TRASH_RETENTION_DAYS`（既定30, 0 で無効）。**削除は必ず二段階、dry_run 既定 true** | M | B5 |
| **B11** | ミドルウェア一式 | サイズ / タイムアウト / 並行数 / レートいずれも無し。A9 を入れるなら必須 | `RequestBodyLimitLayer`（API 1MB / upload 64MB を `route_layer` で分離）、`TimeoutLayer`(30s、**`/storage/` は除外**)、`ConcurrencyLimitLayer`、`tower_governor`（login は厳しめ） | **S** | — |
| **B12** | メトリクス `/metrics`（Prometheus） | headless 運用では UI での異常検知が効かない。**「取り込みが止まっている」ことに気づく唯一の目** | `import_total{outcome}` / `parse_total{source_tool,has_metadata}` / `http_requests_total` / `storage_bytes_total` / `job_running{kind}` | S〜M | — |
| **B13** | 統計をプロセス内キャッシュ → 集計テーブル | 10分 TTL のプロセス内キャッシュ。加えて stats / catalog は `jsonb_array_elements(loras)` の**全表スキャンを毎回**（`min_count` の HAVING はスキャン後） | 即効: `MATERIALIZED VIEW mv_lora_usage / mv_model_usage` + B5 で `REFRESH CONCURRENTLY`（既存 SQL の FROM 差し替えのみ）。本格: `image_loras(image_id, lora_name, weight, position)` に正規化（取り込み・再解析の両方から同期が要るので B4 とセットで設計） | M | B1・B5 |
| **B14** | `GET /api/version` / `/api/config` ★安いのに効く | Falcon が「今つないでいる PromptBox は何者か」を知る手段がゼロ（`GET /` はハードコードの `0.1.0-rs` のみ） | `{version, git_sha, built_at, schema_version, parser_version, features:[], limits:{max_per_page, bulk_max_ids, max_upload_bytes}, thumbnail_sizes:[]}` | **S** | — |
| **B15** | エラー本文の統一 + 適切なステータス | 不正 tsquery で **500** → Falcon 側でリトライ対象になり無駄な再送を生む | `{error:{code, message, param, details}}` に統一（`src/error.rs`）。tsquery 構文エラーは 400（C1 で構文エラー自体が消えるのでセット） | S | — |

> **B9（claim テーブルによるクラッシュセーフ取り込み）は見送り推奨。** 現状の実害が観測されておらず、**B10 の GC で実質カバーできる**。

### 観点③ 検索・発見の強化

| ID | 案 | 何を解決するか | 実装の要点 | 工数 | 依存 |
|---|---|---|---|---|---|
| **C1** | **全文検索の作り直し** ★方向性の再定義 | 「english → japanese に変える」は**誤答**。AI 画像のプロンプトは自然文ではなく**カンマ区切りのタグ列**（`1girl, solo, (masterpiece:1.2)`）。英語ステミングは意図せぬ一致を生み、日本語は英語辞書で分割不能 | ① `to_tsvector('simple', ...)` に変更（ステミング・ストップワード無し＝タグ列に正しい。**日本語も空白 / カンマで区切られていればそのままトークン化される**）② `websearch_to_tsquery` に変更 → **`!` `\|` `(` の 500 が構文的に消え**、`"句"` と `-除外` と `or` が無料で付く ③ CJK / 短語のために `gin_trgm_ops` の trigram index を併用し `ILIKE` フォールバック ④ 生成列 `search_vector tsvector GENERATED ALWAYS AS (...) STORED` + GIN。**PGroonga / pg_bigm は最後の手段**（Postgres イメージのビルドが必要で運用コストが跳ねる） | M | B1 |
| **C2** | **プロンプトのトークン化テーブル** ★③④共通の土台・最大レバレッジ | プロンプトが巨大な TEXT 1本。「`masterpiece` を含む」が `masterpieces` にも当たり、「重み `(detailed:1.4)` と評価の相関」は問えず、「よく一緒に使うタグ」も出せない。**C1 を直してもテキスト検索である限り全部できない** | `image_prompt_tokens(image_id, polarity, position, token, weight)`。トークナイザ（新 `src/prompt/mod.rs`）: `,` 分割 → `(tag:1.2)` / `((tag))` / `[tag]` の重み記法 → `<lora:name:0.8>` → `BREAK` / `AND` 制御語 → 小文字化・アンダースコア正規化。**純粋関数なのでユニットテストが厚く書ける**（parser と同じ扱い）。API: `?prompt_tags=1girl,solo&exclude_prompt_tags=nsfw`（**完全一致タグ検索**）、`GET /api/prompt-tags?sort=count\|rating\|lift` → `{token, count, avg_rating, lift, first_seen, last_seen}`。**`user_tags` 用の `/api/tags` が count 無しの `Vec<String>` である問題も同じレスポンス形に揃えて解消**。⚠️ 正規化ルールを後から変えると全再生成なので `tokenizer_version` 列を最初から持つ | M〜L | B5・B4 |
| **C3** | **知覚ハッシュ（pHash）+ 近傍検索** | SHA-256 は1バイト違えば別物 → リサイズ版・再エンコード版・微差の hires fix が全部別画像として溜まる | `images.phash BIGINT`。**サムネ生成と同じデコード結果から計算＝追加デコード不要でほぼタダ**（`src/media/mod.rs`）。既存分は B5 でバックフィル（原本再読み込みが要るので数万枚なら数時間）。PG16 なら `bit_count(phash # $1) <= 8` の全表スキャンで数万行なら実用。API: `GET /api/images/{id}/similar?max_distance=8`、`GET /api/duplicates/near?group=true`。⚠️ AI 画像は「同構図で微妙に違う」が大量にあるので閾値を返り値に含め調整可能に | M | B5 |
| **C5** | **filters DSL の一元化** | スマートフォルダの `filters` がサーバで評価されない。ただし「件数 API を付ける」より **「DSL 評価器を1つ作り、preset / smart-folder / 検索 / エクスポート / 一括操作が全部同じ DSL を食う」**設計が正しい | **`impl From<&SearchFilters> for SearchParams` の1関数が価値の大半**。これで `GET /api/smart-folders/{id}/images`・`/count`（一覧に `item_count`）、`POST /api/images/search`（GET のクエリ長制限回避）、`POST /api/bulk/update {filters, update}`（id 列挙不要の条件一括）、`export?preset_id=` が一気に開く。**A3 の warnings があると「保存されていた filters のうち評価されなかったキー」を返せて安全** | S〜M | — |
| **C6** | ファセット（現在の絞り込み文脈での候補 + 件数） | `/api/stats/*-for-filter` は**ライブラリ全体**の集計で、「今 `model_type=pony` に絞った状態で選べる sampler」に答えられず**結果0件のフィルタを提示してしまう** | `GET /api/images/facets?<検索と同じ全パラメータ>&facets=model_name,sampler_name,...`。`push_filters` を再利用して各列 `GROUP BY`、`tokio::try_join!` で並列 + `LIMIT 30` | M | C5・B13 |
| **C7** | フィルタ DSL v2（AND/OR/NOT 入れ子） | 全フィルタが暗黙 AND のフラット構造。「(モデル A or B) かつ LoRA C を使っていない」が表現不能。Falcon の `smart_folder_conditions` との対応づけにも効く | `POST /api/images/search {"where":{"op":"and","children":[...]}}` を `QueryBuilder` で再帰構築。**フィールド名は必ずホワイトリスト enum、値は `push_bind`**。フラット API を DSL v2 の糖衣として実装し内部を一本化 | M〜L | C5 |
| **C8** | 未評価消化キュー `GET /api/images/triage` | Quick Rate のサーバ側インテリジェンス。実際に欲しいのは「同設定の連番20枚を全部」ではなく「似た画像はクラスタ代表1枚、モデルの偏りを均して」 | `?strategy=recent\|diverse\|cluster_head`。cluster_head は C3 の pHash クラスタ代表のみ、diverse は model_name / prompt_hash でラウンドロビン。**Falcon の Quick Rate 画面が薄いラッパで済む＝責務線引きの好例** | S〜M | C3 |
| **C9** | セッション / バースト検出 | 「昨日の夜に試した一連の実験」を単位として扱えない | `LAG()` ウィンドウ関数で `created_at` 差が閾値内の塊をグループ化 → `{started_at, ended_at, count, models, top_tokens, avg_rating}`。純 SQL | M | — |
| **C10** | サンプリング取得 `?sample=50&sample_seed=` | 「ランダムに50枚」「高評価からランダムに壁紙」が offset では表現できない | `ORDER BY md5(id::text \|\| $seed) LIMIT n`（大規模なら `TABLESAMPLE SYSTEM`） | S | — |
| **C4** | 埋め込みベクトル検索（pgvector + CLIP/SigLIP） | pHash は画素の類似、埋め込みは**意味の類似**で別物。「この絵に似た雰囲気」「プロンプトに無い語で探す」 | `image_embeddings(image_id, model, embedding vector(768))` + HNSW。推論は**別コンテナのサイドカー**（`EMBEDDING_SERVICE_URL`）が現実的（モデル配布・更新が独立する）。**C3 の時点で `image_features` テーブルとして設計しておくと後で楽** | L | C3・B5 |

### 観点④ 生成ナレッジの活用（PromptBox の存在意義）

> **設計方針: PromptBox は「提案」しない。「提案の材料になる、統計的に正直な数字」を API で出す。** 判断・UI・LLM 呼び出しは Falcon なり外部の仕事。これが責務の線。

| ID | 案 | 何を解決するか | 実装の要点 | 工数 | 依存 |
|---|---|---|---|---|---|
| **D1** | プロンプトタグの効果分析（**lift + 信頼区間**） | TODO.md【既出】「高評価画像のプロンプト自動抽出」の統計的に正しい実装。単純な「高評価画像に多いタグ」は**単に頻出なだけのタグ**（`masterpiece`, `best quality`）が上位を占めて役に立たない | `GET /api/knowledge/prompt-tokens?min_count=20&sort=lift`。`lift = 含む画像の平均評価 − 含まない画像の平均評価`、`ci95` は Welch の t 区間。**信頼区間を返すのが要点**（n=5 の「lift +1.5」を上位に出すと嘘になる）。交絡（そのタグは特定モデルでしか使わない）対策に `?control=model_name` の層別平均。**「相関であって因果ではない」を API doc に明記** | M | C2 |
| **D2** | **汎用 affinity マトリクス** | TODO.md【既出】「モデル互換性 DB」「モデル×LoRA×サンプラー相性」。**個別 API を3本作らず1本に畳む** | `GET /api/knowledge/affinity?rows=model&cols=lora&metric=avg_rating&min_count=10` → `{cells:[{r,c,count,avg_rating,lift,ci95}], row_baseline, col_baseline}`。軸に `model\|lora\|sampler\|scheduler\|model_type\|prompt_token\|steps_bucket\|cfg_bucket\|source_tool` を許可 → **これ1本で既出の相性系が全部カバーされる**。⚠️ **選択バイアス**（試した組み合わせしか無い・良さそうだから試した）→ `count` と `ci95` 必須、`min_count` 既定を高め | M | B13・C2 |
| **D3** | 推奨設定（自分の実績 × CivitAI の合成） | TODO.md【既出】。既存 `src/civitai/mod.rs` は CivitAI の推奨を取れるが**自分の実績と突き合わせていない** | `{from_library:{steps:{p25,median,p75,n}, top_samplers:[...]}, from_civitai:{...}, agreement:{...}}`。⚠️ 名寄せ（`extract_base_model_name` は正規表現ヒューリスティック）→ **確度を添えて「参考」として返す** | M | D2 |
| **D4** | LoRA ウェイト最適点 | TODO.md【既出】「LoRA ウェイトと評価の相関」 | 既存 `lora_detail` の `filtered` サブクエリに `width_bucket(lora_weight, 0, 2, 20)` を足すだけ → `[{weight_range:"0.6-0.7", count, avg_rating, ci95}]`。`min_count` 必須（n が小さいビンだらけになる） | **S** | — |
| **D5** | ネガプロ定型ブロックの**自動抽出** | TODO.md【既出】「ネガティブプロンプトライブラリ」。**手で目的別に整理するライブラリは作った瞬間に陳腐化する** → 自分の履歴から実際に使っている定型を検出するほうが価値が高い | C2 のトークン列そのもののハッシュで頻度集計 → `{presets:[{tokens, count, avg_rating, used_with_models, first_seen, last_seen}]}`。実質「自分が固定で使っているネガプロの棚卸し」。**完全一致の集計だけで実用上ほぼ足りる**（n-gram マイニングは不要） | S〜M | C2 |
| **D6** | **ワークフロー正規化・指紋・カタログ** ★最大の未活用資産 | TODO.md【既出】「ワークフロー保存・管理」「類似ワークフローのグループ化」。**だがデータは既に全部 `raw_metadata` にある。必要なのは保存機能ではなく索引付け** | 正規化（新 `src/workflow/mod.rs`）: ノード ID を位相順で再採番 → `seed` / `text` / `filename_prefix` 等の可変入力をマスク → 安定シリアライズ → SHA-256。**2段階のハッシュ**が要点: `workflow_hash`（厳密＝接続とノード種別と固定パラメータ）と `workflow_shape_hash`（class_type の多重集合のみ＝ノードを1個足しても同じ系統に留まる）。`workflows(hash, shape_hash, node_count, class_types[], graph, first_seen, last_seen, image_count)`。**副産物: `class_types` の全体集計＝「自分が実際に使っているカスタムノード拡張の棚卸し」**（ComfyUI 再構築・拡張更新時に効く）。⚠️ ComfyUI 限定（A1111 / NovelAI はグラフ無し → NULL）。`workflow_normalizer_version` 列を持ち変更時は B5 で再計算 | M〜L | B1・B5 |
| **D7** | ワークフロー diff | TODO.md【既出】の API 化 | `{added_nodes, removed_nodes, changed_inputs:[{node,key,before,after}], rewired}`。表示は Falcon。**ノード ID を保持したまま比較すれば実用上十分**（同ワークフローの派生比較が主用途で、一般のグラフ同型判定は不要） | S〜M | D6 |
| **D8** | **再生成レシピ API** `GET /api/images/{id}/recipe` ★責務分割の要 | TODO.md【既出】「ComfyUI への直接送信」。**だが送信は PromptBox の仕事ではない** — ComfyUI に HTTP を投げ始めると稼働・URL・認証まで抱え、取り込みエンジンの境界が崩れる | 正しい分割は「**PromptBox はそのまま POST できるペイロードを作るところまで**、送信は Falcon or クライアント」。`?target=comfyui` は `raw_metadata.prompt` を override 適用して `{"prompt":{...}}` で返すだけ（**実質シリアライズ＋値差し替えで安い**）。`target=a1111`（parameters 文字列 or txt2img JSON）/ `target=params`（ツール非依存）も。**`required_class_types` と `required_models` を添えて送信側が事前検証できるように** | M | D6 |
| **D9** | 系譜（lineage）の自動推定 | TODO.md【既出】「生成実験タイムライン」「分岐点検出」は素直にやると ComfyUI 側に experiment_id を仕込む改造が必要で実現性が低い。**ComfyUI を一切改変せずに実現する道がある** | 同 `workflow_shape_hash` かつ `created_at` 差が閾値内かつ**差分パラメータが1〜2個だけ**のものを親と推定 → `image_lineage(child_id, parent_id, delta, confidence)`。`GET /api/images/{id}/lineage`（祖先 / 子孫ツリー + 各エッジの delta）、`GET /api/experiments` → `{root, size, params_explored:["seed","cfg_scale"], best_child}`。⚠️ 推定なので誤リンクする → `confidence` を返し閾値を設定可能に | L | D6・C9・B5 |
| **D10** | 生成プリセット | TODO.md【既出】「生成スタイルプリセット」 | `generation_presets(...)` + `POST /api/generation-presets/from-image/{id}` — **既存画像から1クリックで起こす**（手入力させない）。`GET .../recipe?target=comfyui` で D8 に合流。import / export（JSON）。⚠️ **「状態を持つ機能」なので責務線引きに反する疑い** → Falcon 側に持たせるか要判断（ただしレシピ変換には ComfyUI グラフの知識が要るので PromptBox 側が自然ではある） | M | D8 |
| **D11** | メタデータ差分 API `POST /api/images/compare` | TODO.md【既出】「A/B 比較ビュー」。**画像を並べるのは Falcon の仕事。PromptBox が出すべきは差分の構造化** | `{same, different:[{field, values}], prompt_diff:{added, removed, reordered}, lora_diff}`。prompt_diff は C2 のトークン列の集合差分。**2枚でなく N 枚に一般化**（XYZ グリッド相当の比較になる） | S〜M | C2 |
| **D12** | シード変種グルーピング | TODO.md【既出】「シード変種エクスプローラー」 | `images.settings_hash` = md5(model, 正規化 pos/neg, sampler, scheduler, steps, cfg, loras) — **seed を除く全設定**。index 一発で `GET /api/images/{id}/variants`。⚠️ JSONB を触る生成列は IMMUTABLE 制約に引っかかる → **トリガかアプリ側で計算して普通の列に書く** | S〜M | C2 |
| **D13** | モデル / LoRA の棚卸し（未使用検出） | catalog は「使った実績」しか見えない。**「導入したのに一度も使っていない LoRA」がディスクを食っている**、逆に**「画像はあるがファイルが消えた＝再生成不能」**も検知できない | `MODELS_PATH` / `LORAS_PATH`（読み取り専用）を設定に追加 → `GET /api/catalog/inventory` → `{files:[{path,size,mtime,image_count,avg_rating}], unused, missing_but_used}`。名寄せは**完全一致 → basename 一致 → ハッシュ一致の3段フォールバック**、不一致はそのまま報告 | M | — |
| **D14** | Gelbooru カテゴリのトークン付与 | `src/http/gelbooru.rs` は現状「タグを検索するだけ」の孤立機能。C2 のトークンに category（general / character / copyright / artist）を付ければ「キャラ名で探す」「絵師タグの効きを分析」が可能に | `prompt_token_meta(token, gelbooru_category, post_count, aliases, fetched_at)`。新規トークン出現時に非同期問い合わせ（B5 のジョブ、レート制限考慮、TTL 30日） | M | C2・B5 |
| **D15** | **ナレッジダイジェスト** `GET /api/knowledge/digest?format=markdown` ★コスパ最良 | TODO.md の「AI 提案機能」群を **PromptBox 内で AI 実装しないための出口**。LLM 推論を backend-rs に持ち込むとモデル配布・API キー・コスト・レイテンシを全部抱える | 代わりに「**LLM にそのまま食わせられる形にライブラリの知見を要約して吐く**」だけにする（概況 / 高評価モデル top5（n 付き）/ 相性 lift 上位 / 推奨パラメータレンジ / 効くトークン・避けるトークン / よく使うネガプロ）。Claude や ChatGPT に貼れば「次に何を試すか」が返る。**D1/D2/D3 の集約に過ぎないので S** | **S** | D1・D2・D3 |
| **D16** | クラウド複製 | TODO.md【既出】「クラウドバックアップ」 | **OAuth（Google Drive）はやらない**。`object_store` で抽象化済みなので **S3 互換の第2ターゲットへの複製ジョブ**が最小コスト（R2 / B2 / Wasabi は全部 S3 API）。`BACKUP_S3_*` + B5 のジョブで差分同期。⚠️ **長期保存は Falcon（DAM）の責務**なので、PromptBox は `GET /api/admin/export/full`（NDJSON ダンプ）だけ提供して撤退するのが筋 | M | B5 |

---

## 4. 凍結・撤去候補（足すより減らす）

headless 化した以上、**新機能を追加しない**方針を明示すべきもの:

| 対象 | 判断 | 理由 |
|---|---|---|
| `showcases` / `showcase_images` | **凍結**（改修も新機能もしない） | Falcon の `collections` が上位互換。将来は移行して撤去 |
| `search_presets` / `smart_folders` の**保存** | **凍結**。ただし C5 の**評価エンジンは強化** | 保存先は Falcon が正しい。PromptBox は filters を評価するだけ |
| `/api/export/{metadata,prompts}` | 現状維持 | Falcon の proxy allow-list に載っているので壊さない |
| `src/http/gelbooru.rs` | **D14 に統合するか削除** | 現状は孤立した検索プロキシ。単体では headless エンジンの責務外 |
| `thumbnail_url` vs `thumbnail_path` の不統一 | **A2 の manifest 作成時にまとめて是正** | 単独でやると破壊的変更。契約を明文化するタイミングで直す |
| `include_deleted` の命名と挙動の乖離 | **`status=active\|deleted\|all` を追加**、旧パラメータは A3 の deprecated warning で誘導 | 破壊せず移行 |

---

## 5. 優先度案：これだけやるなら、何を、どの順で

### すぐ（P0）— 土台と止血。これ無しに他が乗らない

| # | 項目 | 工数 | なぜ今か |
|---|---|---|---|
| 1 | **B1** スキーマの単一の真実（`db/init` → `migrations` 一本化） | M | 以下ほぼ全部が新列 / 新テーブルを伴う。**ここが詰まると全部詰まる** |
| 2 | **B2** `#[sqlx::test]` 統合テスト基盤 | M | B1 の直後でないと二度手間。SQL ロジック全無検証の現状は危険 |
| 3 | **C1** 検索の作り直し（`simple` + `websearch_to_tsquery` + trigram） | M | 500 が消え、日本語が引ける。単発で効果が最大 |
| 4 | **B14** `/api/version`・`/api/config` | S | 半日。Falcon がバージョン差を検知できる |
| 5 | **A3** `warnings[]` + `?strict=true` | S | Falcon が実際に事故った箇所（Plan 222）の再発防止。非破壊 |
| 6 | **A2(a)** ルートマニフェスト + Falcon 側の契約テスト | S | Falcon のコードに「404 でしか気づけない」と書いてある。放置しない |
| 7 | **B11** ミドルウェア（サイズ / タイムアウト / 並行 / レート） | S | A9 の前に必須。1日 |
| 8 | **B5** 軽量ジョブ基盤 | M | この後の7〜8機能が全部これに乗る。**1回作って7回使う** |
| 9 | **A1** 変更フィード + トゥームストーン | M | Falcon の増分同期が初めて成立する。C-2 運用の核心 |

**狙い**: Falcon との契約を機械化し（4,5,6）、スキーマとテストの土台を作り（1,2）、非同期処理の共通機構を用意して（8）、Falcon が増分で吸えるようにする（9）。C1 だけは独立して効果が大きいので混ぜている。

### 次（P1）— headless エンジンとしての差別化

10. **B4 + B3** 再解析 + `parser_version` + リグレッション検知 ← §1② の発見が直接効く
11. **B6 + B7** 取り込み可観測性 + 失敗ファイル再投入
12. **C5** filters DSL 一元化（スマートフォルダのサーバ解決を含む）
13. **C2** プロンプトのトークン化テーブル ← **④のほぼ全部の前提。④重視なら 10 より前でもよい**
14. **D1 + D5** タグ効果分析（lift + CI）+ ネガプロ定型抽出
15. **C3** pHash + 近傍検索・近似重複
16. **A4 + A5** API キー + `/storage/` 保護・Range・派生サイズ
17. **B10** 孤児 GC + ゴミ箱パージ + 整合レポート
18. **D2 + D3 + D4** affinity マトリクス + 推奨設定 + weight カーブ
19. **B13** 統計のマテビュー化 → 20. **C6** ファセット
21. **A8 + A6** keyset ページング + batch 取得・スパースフィールド
22. **B12** `/metrics` ／ 23. **D15** ナレッジダイジェスト ／ 24. **C8** 未評価消化キュー

**狙い**: 10〜11 で「パーサを持つエンジン」として自立（直せる・遡及適用できる・見える）、13〜14 + 18 で「生成ナレッジ API」という Falcon に無い価値を確立、15〜17 で運用の穴を塞ぐ。

### いつか（P2）

25. **D6 + D7** ワークフロー正規化・指紋・カタログ・diff
26. **D8 + D10** レシピ API + 生成プリセット
27. **A9** アップロード API
28. **A7** 同期状態 `image_sync`（※先に運用決定）
29. **C9 + D12 + D11** セッション検出 + シード変種 + 比較差分
30. **C7** フィルタ DSL v2 ／ 31. **B8** 複数・再帰監視
32. **D13 + D14** モデル棚卸し + Gelbooru カテゴリ付与
33. **D9** 系譜自動推定 ／ 34. **C4** 埋め込み検索 ／ 35. **A10** Webhook・**D16** クラウド複製 ／ 36. **A2(b)** OpenAPI 完全生成

---

## 6. 着手前に決めるべき2点（技術ではなく運用の決定）

1. **評価（rating / user_tags / memo）のマスタはどちらか。**
   PromptBox がマスタなら A1 + A7 で一方向同期。Falcon がマスタなら PromptBox 側の PATCH / bulk API は**縮小**し、A1 は「新規画像の通知」だけになる。**この決定で観点①の設計が半分変わる**（`docs/12_falcon_integration.md` §8/§9 の宿題そのもの）。
2. **`showcases` / `search_presets` / `smart_folders` を Falcon に寄せるか。**
   寄せるなら C5 は「評価エンジン API」だけでよく CRUD は撤去対象。寄せないなら件数・一覧 API まで作り込む必要がある。

---

## 7. 実装時に触る主要ファイル（参考）

| ファイル | 関わるアイデア |
|---|---|
| `backend-rs/src/image/mod.rs` | `push_filters` / `list` / `neighbors` — C1・C5・C6・C7・A8 が集中。**`neighbors()` の行値比較は A8 の keyset にそのまま転用可** |
| `backend-rs/src/worker/mod.rs` | `import_image` / `insert_image` / `classify_failure` — B6・B7・B8・A9・C3・D6 の取り込み時計算。A9 のため `&Path` 前提をバイト列版へ切り出すリファクタが要 |
| `backend-rs/src/http/mod.rs` | ルータ定義 — A2 のマニフェストはこの配列の宣言的テーブル化で実現。B11 のレイヤ追加も |
| `backend-rs/migrations/` と `db/init/01_init.sql` | **B1 の対象。この2つの関係を解消しないと P0 以降が進まない** |
| `backend-rs/src/parser/mod.rs` | `ParsedMetadata` / `parse()` — B3・B4（`parser::VERSION` と raw_metadata → png_info アダプタ）、D6・D8 の起点 |
| `Falcon/gateway/internal/infrastructure/promptbox/proxy_routes.go` | **A2 の対向側**。契約テストはここに書く |

---

## 関連ドキュメント

- `docs/12_falcon_integration.md` — 統合方針（案 A/B/C）と未確認事項。§6 の判断は本ドキュメントの前提
- `docs/11_rust_rewrite.md` — Rust 移行の残作業
- `TODO.md` — 「AI 提案機能(未精査)」は本ドキュメントの D1〜D16 に具体化した
