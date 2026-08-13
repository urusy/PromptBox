# 設計: グリッド画像の取り込みと専用画面（issue #41）

作成日: 2026-08-13 / 対象: [#41](../../issues/41) / ステータス: 設計完了・未実装

## 要求

1. AI 生成時のグリッド画像（A1111/Forge の XYZ plot 等）も取り込めるようにする。
2. グリッド画像は通常ギャラリーとは**別の一覧画面・詳細画面**で扱う。
3. グリッド詳細画面から、**そのグリッドを構成する画像へ遷移**できるようにする。

## 現状の観測（2026-08-13）

### なぜ今は取り込まれないのか（真因）

ワーカーがファイル名パターンで**取り込み前に隔離**しているため。パーサ・検索は既にグリッド対応済み。

1. `config.rs:97` — `IMPORT_SKIP_PATTERNS` 既定 `xyz_grid`。
2. `worker/mod.rs:197-206`（`process_path`）— `should_skip`（`worker/mod.rs:139-149`、
   小文字化したファイル名への **contains 一致**）に該当すると `import/skipped/` へ隔離し、処理しない。
3. compose は `IMPORT_SKIP_PATTERNS=${IMPORT_SKIP_PATTERNS:-xyz_grid}`
   （`docker-compose.yml:54` / `docker-compose.prod.yml:51`）。
   **`:-` のため .env で空文字を設定しても既定に戻る = 現状は環境変数だけでは無効化できない罠**。

### 既にある資産（作らなくてよいもの）

| 層 | 実装 | 場所 |
|---|---|---|
| パーサ | `Script: X/Y/Z plot` → `model_params.is_xyz_grid=true`、**`xyz_{x,y,z}_{type,values}` も保存済み** | `parser/a1111.rs:143-160` |
| 検索 | `is_xyz_grid` フィルタ（`model_params->>'is_xyz_grid'`） | `image/mod.rs:271-277`、`http/images.rs:61,105` |
| prev/next | `neighbors()` は検索と同じ `push_filters` を共有 → `is_xyz_grid=true` を渡せばグリッド内巡回になる | `image/mod.rs` |
| 生成パラメータ列 | `model_name / sampler_name / steps / cfg_scale / seed` は**専用列**（大半に index あり） | `migrations/20260711000000:34-39,71-74` |
| 重複対策 | `file_hash` 一致で `duplicated/` へ | `worker/mod.rs:256-265` |
| フロント前例 | lazy ページ + 専用一覧/詳細のパターン（Showcase / Model / LoRA）、詳細ルートは `image/:id` | `App.tsx:12-23,54` |

### 制約

- 定期スキャンは import ルート直下のみ（`worker/mod.rs:125-136`、NonRecursive）
  → `skipped/` 配下は再走査されない。既存の隔離ファイルは **mv で戻せば拾われる**。
- サムネ生成は原本を**フルデコード**（`worker/mod.rs:289-299`）。XYZ グリッドは巨大になりうる
  （例: 10×8 の 1024px 格子 ≈ 84MP → RGBA 展開で ~336MB。本番コンテナはメモリ 1GB 制限）
  → **OOM ガードが必要**。
- グリッド PNG の parameters はベース値（1枚目のセル相当）+ 軸定義であり、
  **構成画像との明示的なリンク（ID 参照）はどこにも存在しない** → 構成画像はヒューリスティックで推定する。

---

## Part 1: 取り込み — 「スキップ」から「タグ付けして取り込む」へ

### 1-1. 既定でスキップしない

- `config.rs`: `IMPORT_SKIP_PATTERNS` の既定を `xyz_grid` → **空**へ（機構自体は汎用スキップとして残す）。
- compose 2 ファイル: `${IMPORT_SKIP_PATTERNS:-xyz_grid}` → `${IMPORT_SKIP_PATTERNS:-}`。
- `.env.example:63` のコメントを更新（「グリッドを取り込みたくない場合は `xyz_grid` を設定」と反転）。
- `split_csv` は空文字 → 空 Vec（`config.rs:246-252`）なので追加実装不要。

### 1-2. ファイル名によるグリッド判定の補完（新設 `IMPORT_GRID_PATTERNS`）

パーサの `Script` 検出はメタデータが載っている場合しか効かない
（PNG info 埋め込み無効・再保存だと `has_metadata=false`）。ファイル名でも補完タグ付けする:

- `config.rs`: `import_grid_patterns: Vec<String>`、既定 **`xyz_grid,^grid-`**。
  パターンは既定で部分一致だが、**先頭に `^` を付けると先頭一致**になる
  （2026-08-13 追記: `grid-` を部分一致にすると `my_grid-test.png` のような通常画像まで
  グリッド扱いになる。A1111 のバッチグリッドは必ず `grid-0000.png` と**命名される**ため、
  先頭一致で捕捉率を落とさずに巻き添えだけ消せる）。
- `worker/mod.rs::import_image`: `parser::parse`（`worker/mod.rs:277`）の直後、
  ファイル名がパターンに一致し、かつ `parsed.model_params` に `is_xyz_grid` が無ければ
  `is_xyz_grid=true` を挿入。パーサが既に true にしていれば何もしない。
- 判定関数は `should_skip` と同形の純関数として切り出し、ユニットテストを書く。

### 1-3. 巨大画像対応 — 2 段構えのサムネイル生成（決定②: 上限引き上げでは対応しない）

**前提となる制約**: `thumbnail_path` は NOT NULL（`migrations/20260711000000:27`）なので
「サムネ無しで取り込む」逃げ道は無い。本番 backend-rs はメモリ **1GB 制限**
（`docker-compose.prod.yml:89-93`）なので、フルデコード（RGBA で 4 byte/px）は
**約 200MP が物理的な限界**。「もっと大きなものも対象にする」には方式自体を変える:

| 経路 | 条件 | 方式 | メモリ |
|---|---|---|---|
| 通常 | `w*h ≤ IMPORT_FULL_DECODE_MAX_PIXELS`（既定 150MP） | 現行どおりフルデコード + Lanczos3（`media/mod.rs:38-53`） | px × ~4B |
| **大型** | 超過 かつ PNG（非インターレース） | **ストリーミング縮小**: `png` クレート（依存導入済み）の `next_row()` で走査線を逐次読み、面積平均で縮小バッファへ畳み込む → WebP encode。**全画素を一度にメモリへ載せない** | 数 MB（走査線数本 + 300px 幅の累積バッファ） |
| 対象外 | 超過 かつ 非 PNG / インターレース PNG | `failed/` へ隔離（理由を明示）。実運用のグリッドは A1111/ComfyUI とも PNG 出力が既定なので実害は限定的 | — |

- `image_dimensions`（ヘッダのみの軽量読み取り、`worker/mod.rs:267`）が既にデコード前に
  あるので、そこで経路を分岐する。
- 品質: 300px サムネへの大縮小では面積平均（box filter）と Lanczos3 の差は視認困難。許容する。
- 大型経路ではファイル全体を `bytes` に読む現行実装（`worker/mod.rs:290`）も避け、
  ハッシュ・アップロードともストリーム/パスベースで扱う（object_store の put は
  マルチパート対応。実装時に `put` の分割要否を確認）。
- **`IMPORT_MAX_PIXELS` は安全弁として残す**（既定 **2_000_000_000 = 2GP**、env で調整可）。
  ストリーミングによりメモリは無関係になるため、これは異常ファイル（デコーダ爆弾・
  破損ヘッダ）で CPU を延々消費しないためのガード。超過は `classify_failure` で
  **Permanent** に明示分類し `failed/` へ（専用エラー型かメッセージマーカーをマッチ腕へ追加）。

### 1-4. 既存 `skipped/` ファイルの救済（本番 NAS 運用手順）

1. 新イメージをデプロイ（DB マイグレーション無し・スキーマ変更無し）
2. NAS 上で `mv import/skipped/*.png import/`（定期スキャン 30 秒毎が拾う。コピーでなく mv —
   `created_at` は birthtime/mtime 由来のため）
3. 既取り込み分は `file_hash` 重複で自動的に `duplicated/` へ → 手動整理不要
4. `docker compose logs backend-rs` で `imported` / `quarantined` 件数を確認

---

## Part 2: 構成画像の推定 — `GET /api/images/{id}/grid-members`

### 2-1. マッチングの原理

グリッド行の `model_params` には**ベース値 + 軸定義**（`xyz_x_type` / `xyz_x_values` 等）がある。
構成画像は「軸に使われた列は軸値のどれか、それ以外はグリッドと同値、生成時刻が近い」で絞る:

| 条件 | 内容 |
|---|---|
| 対象外 | `is_xyz_grid` が true の行、`deleted_at IS NULL` 違反 |
| 不変条件 | `{seed, steps, cfg_scale, sampler_name, model_name}` のうち**軸に使われていない列**は グリッド行の値と一致（グリッド側が NULL の列は条件から外す） |
| 軸条件 | 軸タイプが下表でマップできる列は `IN (軸値リスト)` |
| 時間窓 | `created_at ∈ [grid.created_at − window, grid.created_at + margin]`。既定 window=24h / margin=10min（グリッドはセル生成後に保存されるが、birthtime の揺れに備える）。`?window_hours=` で調整可 |

**軸タイプ → 列マッピング**（v1 対応分。純関数としてユニットテスト対象）:

| A1111 軸タイプ | 列 |
|---|---|
| `Seed` | `seed` |
| `Steps` | `steps` |
| `CFG Scale` | `cfg_scale` |
| `Sampler` | `sampler_name` |
| `Checkpoint name` | `model_name` |
| `Prompt S/R` ほか上記以外 | **未対応** — 軸条件は掛けず、不変条件 + 時間窓のみで推定し `warnings[]` で通知（A3 準拠、`code: "unsupported_axis_type"`） |

- SQL は不変条件 + 時間窓 + IN 絞り込みまで。**軸値リスト内の位置計算と (z, y, x) 整列は Rust 側**
  で行う（メンバーは高々数十〜百枚。SQL を複雑にしない）。軸値のパースは
  `"5,7,9"` の数値列と引用符付き文字列列の両方に対応。
- `seed` 列に index が無い可能性あり（initial_schema の index 一覧に無い）。42k 行なら実害無しだが、
  実装時に確認し必要なら追加マイグレーション。

### 2-2. API 仕様

```
GET /api/images/{id}/grid-members?window_hours=24
```

- 対象が `is_xyz_grid=true` でなければ 400（既存 `AppError` の形式）。

```jsonc
{
  "grid": { /* ImageListItem */ },
  "axes": {
    "x": { "type": "CFG Scale", "values": ["5", "7", "9"], "column": "cfg_scale" },
    "y": { "type": "Sampler", "values": ["Euler a", "DPM++ 2M"], "column": "sampler_name" },
    "z": null
  },
  "members": [
    { /* ImageListItem */, "position": { "x": 0, "y": 1, "z": 0 }, "axis_values": { "x": "5", "y": "DPM++ 2M" } }
  ],
  "expected_cells": 6,          // |X|×|Y|×|Z|（軸が全てマップできた場合のみ）
  "matched": 6,
  "confidence": "exact",        // exact(=expectedと一致) | partial | heuristic(軸未対応) | none
  "warnings": []                // A3 の warnings[] と同形
}
```

**縮退状態**（必ず実装・テストする）:

| 状況 | 挙動 |
|---|---|
| `has_metadata=false`（ファイル名タグ付けのみ） | `axes: null`、`members: []`、`confidence: "none"` |
| ComfyUI カスタムノード等、軸定義なし | 同上 |
| 軸タイプが未対応 | 不変条件のみで推定、`confidence: "heuristic"` + warning |
| メンバー数 ≠ 期待セル数 | `confidence: "partial"`（削除済み・取り込み漏れで普通に起きる） |

### 2-3. 実装配置

- 新 `src/image/grid.rs` — 軸マッピング・値パース・位置計算（純関数群）+ メンバークエリ
- `http/images.rs` — ハンドラ、`dto/` にレスポンス型
- `http/mod.rs` — ルート追加 + **`_manifest` の宣言へ追記**
  （A2a の契約テストが「ルータにあるが未宣言」を落とすため同時更新が必須）
- 将来（v2・スコープ外）: 推定結果を `grid_members` リンクテーブルに固定化する B5 ジョブ。
  推定ロジックの改善を遡及適用できる形にするため、v1 ではテーブルを持たない。

---

## Part 3: フロントエンド — 専用一覧・詳細画面

### 3-1. ルーティングとナビゲーション

- `/grids` → `GridsPage`、`/grids/:id` → `GridDetailPage`。
  いずれも lazy ロード（`App.tsx:12-23` の既存パターン踏襲）。
- `MainLayout.tsx` のヘッダー nav に「Grids」リンクを追加（Models / LoRAs の並び）。

### 3-2. GridsPage（一覧）

- データ取得は既存 `GET /api/images` に **`is_xyz_grid=true` を固定**して呼ぶだけ。専用 API 不要。
- `ImageGrid`（仮想スクロール込み）+ ページネーションを再利用。
  フィルタはスリム構成（`q` / `model_name` / 日付 / ソート）で、URL クエリ同期は
  GalleryPage の既存パターンを踏襲。Grid Filter コントロールは出さない（固定のため）。
- クリック → `/grids/:id`。

### 3-3. GridDetailPage（詳細）

構成（上から順に）:

1. **グリッド画像** — `ProgressiveImage` 再利用、クリックで拡大（既存 DetailPage の部品を共通化して
   再利用。部品の抽出単位は実装時に決める）
2. **軸情報テーブル** — X/Y/Z の type と values（`model_params.xyz_*` から）
3. **メタデータ・評価** — 既存 DetailPage 相当（グリッドも images の行なので評価・タグはそのまま効く）
4. **Members セクション** — `GET /api/images/{id}/grid-members` の結果を
   (z, y, x) 順のサムネイル格子で表示。キャプションに `axis_values`。
   **クリック → `/image/:id`（既存の通常詳細画面）** — これが issue の「構成画像へ遷移」。
   `confidence` バッジと、`partial` 時は「n/m 枚が見つかりました」、`none` 時は
   「メタデータが無いため構成画像を特定できません」を表示。
5. **prev/next** — 既存 neighbors API に `is_xyz_grid=true` を渡してグリッド内巡回。

### 3-4. 通常ギャラリーからグリッドを完全分離（決定④）

通常の一覧表示ではグリッド画像を**一切表示しない**。グリッドは `/grids` 専用一覧のみで扱う:

- `GalleryPage` は API 呼び出し時に **常に `is_xyz_grid=false` を付与**（ユーザーが変更する
  手段は設けない）。`/grids` は常に true。3 値の URL マッピングは**不要になり実装が単純化**する。
- `SearchForm` から Grid Filter コントロール（`GRID_FILTER_OPTIONS`、`SearchForm.tsx:25-28,795-`）
  を**撤去**。混在表示は提供しない。
- プリセット / スマートフォルダに保存済みの `is_xyz_grid`（`dto/preset.rs:37`）は
  フロントで**無視**する（フィールド自体は後方互換のため DTO に残す。「Grid Only」を保存した
  プリセットは意味を失うが、グリッドは `/grids` で見られるため許容）。
- バックエンドの `is_xyz_grid` クエリパラメータは**汎用 API として維持**
  （`/grids` が使う。Falcon 等の API 直叩きの互換も保つ）。

---

## テスト計画

| 種別 | 内容 |
|---|---|
| unit（worker） | グリッド判定関数: 一致 / 不一致 / 大文字混在 / パターン空 |
| unit（media） | ストリーミング縮小: フルデコード経路との画素近似比較（小画像で検算）/ 非インターレース判定 / 経路分岐の境界値 |
| unit（config） | `IMPORT_SKIP_PATTERNS` 空文字 → 空 Vec、新設 3 変数の既定値 |
| unit（image/grid.rs） | 軸タイプ→列マッピング / 軸値パース（数値・引用符付き文字列）/ 位置計算・整列 |
| 統合（tests/grid_members.rs 新規） | 不変条件での抽出 / 軸 IN 絞り込み / 時間窓外の除外 / 非グリッドへの 400 / metadata 無し→none / 削除済み除外 / partial 判定 |
| 統合（既存） | `_manifest` 契約テストの宣言更新（更新しないと落ちる）、`image_search.rs` に `is_xyz_grid` フィルタが無ければ追加 |
| フロント | テストファイル無し → `tsc --noEmit` / eslint / build + 実機（一覧→詳細→構成画像→戻る、の動線） |
| ゲート | `cargo clippy --all-targets -- -D warnings` / `cargo test`（DATABASE_URL は localhost:5433） |

## 実装フェーズ（中断に備えた区切り）

1. **Phase 1**: 取り込み（Part 1）— これだけで grids が DB に入り始める → **済（2026-08-13）**
   - `config.rs`: skip 既定を空へ、`IMPORT_GRID_PATTERNS` / `IMPORT_FULL_DECODE_MAX_PIXELS` /
     `IMPORT_MAX_PIXELS` を追加
   - `media/mod.rs`: `MediaError`、`thumb_dimensions`、`create_thumbnail_streaming_png`
     （PNG 走査線 + 面積平均）
   - `worker/mod.rs`: `matches_pattern` 共通化、`is_xyz_grid` 補完、画素上限ガード、
     `store_decoded` / `store_streaming` / `put_file_multipart`、`classify_failure` に MediaError
   - compose 2本 / `.env.example` 更新
   - 検証: `cargo test --lib` **44 passed**（新規6本）、`cargo clippy --all-targets -D warnings` **警告0**
2. **Phase 2**: grid-members API（Part 2）+ 統合テスト → **済（2026-08-13）**
   - 新 `src/image/grid.rs`（軸マッピング / 値パース / `find_members` / `place` / `model_key`）、
     新 `src/dto/grid.rs`、`http/images.rs` にハンドラ、`http/mod.rs` + `manifest.rs` に登録
   - `warnings.rs` に汎用 `note()` を追加（`unsupported_axis_type` / `no_axis_metadata` / `truncated`）
   - **設計からの変更**: `Checkpoint name` 軸は SQL で絞らない（A1111 のドロップダウン表記と
     `images.model_name` が揃わないため）。不変条件から外し、位置決めは Rust 側の `model_key`
     正規化で行う
   - 検証: `cargo test` **全 149 passed**（`tests/grid_members.rs` 新規11本）、clippy 警告0
3. **Phase 3**: 専用画面（Part 3-1〜3-3）→ **済（2026-08-13）**
   - 新規: `types/grid.ts` / `api/grids.ts` / `components/grid/GridMembers.tsx` /
     `pages/GridsPage.tsx` / `pages/GridDetailPage.tsx`
   - `App.tsx` に lazy ルート `/grids`・`/grids/:id`、`MainLayout` に Grids リンク
   - **設計からの変更**: `GridDetailPage` は詳細画面を新規実装せず **`DetailPage` を再利用**し、
     下に `GridMembers` を足す構成にした（DetailPage は 749 行あり、複製すると評価・タグ・
     ライトボックス・ショーケース追加まで二重保守になるため）。そのために
     `DetailPage` へ `basePath` / `listPath`、`ImageGrid` / `ImageCard` へ `linkBase` を追加
   - 一覧の既定レイアウトは justified（グリッドは縦横比がまちまちで切り抜くと読めないため）
   - 検証: `tsc --noEmit` / `eslint --max-warnings 0` / `vite build` すべて通過
4. **Phase 4**: ギャラリーからのグリッド分離（Part 3-4）→ **済（2026-08-13）**
   - `GalleryPage` は API 呼び出し直前に `is_xyz_grid: false` を必ず上書き
     （URL や保存済みプリセットに値が残っていても効かない）
   - `SearchForm` から `GRID_FILTER_OPTIONS` と Grid Filter UI を撤去、
     `filtersMatch` の比較キーと `hasActiveFilters` からも `is_xyz_grid` を除外
   - 検証: フロント3ゲート通過、backend `cargo test` **149 passed** / clippy 警告0
   - **未実施（環境が要る）**: 実機確認（実際のグリッドを取り込んでの目視）、
     本番 NAS での `skipped/` 救済（§1-4 の手順、ユーザー作業）、
     CodeRabbit レビュー（`coderabbit` CLI が未インストールで実行不可）

## 「取り込み対象に増えたのは本当のグリッドだけか」の検証（2026-08-13）

| 論点 | 結論 | 根拠 |
|---|---|---|
| skip 解除で新たに取り込まれるもの | **ファイル名に `xyz_grid` を含むファイルのみ** | 旧 `IMPORT_SKIP_PATTERNS` の既定は `xyz_grid` 単独。構成画像（セル）は `00001-1234567.png` 形式で以前から取り込まれていた |
| セル画像がパーサでグリッド誤判定されないか | **されない** | A1111 `scripts/xyz_grid.py` は `extra_generation_params['Script'] / 'X Values'` を **`process_images(pc)` の後**に代入し、`grid_infotext` の生成にのみ使う。セルの infotext には入らない（一次情報を取得して確認） |
| パーサ判定は今回の追加か | **違う。2025-12-08 から存在**（`ca53fdb`、Python 時代） | 今回のフラグ増加はファイル名判定の分のみ |
| 既存の取り込み済み画像への影響 | **Phase 1 では無し**（`is_xyz_grid` は取り込み時に確定） | Phase 4 で、既にフラグの付いた本物のグリッドが `/grids` へ移るのみ |
| ファイル名判定の巻き添え | **あった → `^grid-` で解消**（§1-2） | `grid-` の部分一致が `my_grid-test.png` 等を拾っていた |

本番 DB でも確認したい場合は、想定外のフラグが無いことを次のクエリで見られる（0 件なら
ファイル名 `xyz_grid` 以外にフラグは付いていない）:

```sql
SELECT count(*) FROM images
 WHERE model_params->>'is_xyz_grid' = 'true'
   AND original_filename NOT ILIKE '%xyz_grid%';
```

## スコープ外として残した点

- **Quick Rate（`/swipe`）とゴミ箱（`/trash`）にはグリッドが出る。** 決定④は「通常の一覧表示」＝
  ギャラリーについての指示であり、評価 UI とゴミ箱は対象外と解釈した。Quick Rate に巨大な
  グリッドが回ってくるのが煩わしければ、同じ1行（`is_xyz_grid: false`）で除外できる。

## 決定事項（2026-08-13 ユーザー判断・全確定）

1. `grid-` を既定タグ付けパターンに**含める**（バッチグリッドもグリッド扱い）。
   → 2026-08-13 追記: **先頭一致 `^grid-` に変更**（上記 §1-2）。捕捉対象は変わらず、
   ファイル名の途中に `grid-` を含む通常画像の巻き添えだけを除いた。
2. 巨大画像は上限引き上げではなく**ストリーミング縮小で対応**（§1-3）。
   PNG なら実質サイズ無制限、`IMPORT_MAX_PIXELS`（2GP）は異常ファイル用の安全弁に格下げ。

3. 時間窓の既定は **24h のまま**（§2-1）。
4. 通常ギャラリーではグリッドを**表示しない（完全分離）**。専用一覧 `/grids` のみで扱う（§3-4）。

**未決事項は無し — 全決定済み、実装着手可能。**

## 工数

M〜L（Phase 1: M ※ストリーミング縮小分増、Phase 2: M、Phase 3: M、Phase 4: S）。
DB マイグレーション無し。
