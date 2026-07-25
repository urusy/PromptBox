# ランブック: 既存 DB へ initial_schema マイグレーションを「適用済み」として記録する

対象: `backend-rs/migrations/20260711000000_initial_schema.sql` を導入したバージョンへ更新するとき
（docs/13 の B1「スキーマの単一の真実」）。**本番 DB を触る作業。実行前に全文を読むこと。**

## なぜ必要か

これまで初期スキーマは `db/init/*.sql` が `docker-entrypoint-initdb.d` 経由で**空ボリュームの初回起動時にだけ**
作っていた。sqlx マイグレーションは `20260711000001_baseline`（`SELECT 1;` の no-op）から始まるため、
**migrations だけでは空 DB を再構成できない**状態だった（CI・統合テスト・DR リストアが成立しない）。

B1 でその内容を `20260711000000_initial_schema.sql` に昇格した。既存 DB にはテーブルが既にあるので、
**このバージョンを「適用済み」として `_sqlx_migrations` に手動で記録しないと、次回起動時に
`CREATE TABLE images` が実行されて失敗し、backend-rs が起動しない。**

## 前提の確認（必須）

```bash
# 1) 現在の記録を確認。20260711000001 のみのはず
docker compose exec -T db sh -c \
  'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT version, description, success FROM _sqlx_migrations ORDER BY version;"'

# 2) スキーマのバックアップ（切り戻し用）
docker compose exec -T db sh -c \
  'pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" --schema-only --no-owner' > schema-before.sql
```

`20260711000000` が既にあれば作業不要（適用済み）。

## ⚠️ 順序を間違えたときの症状と復旧（2026-07-25 に実際に発生）

**手順2の INSERT より先に新イメージをデプロイすると**、backend-rs が
`Error: while executing migration 20260711000000: relation "images" already exists` で
exit 1 し、数秒間隔で**再起動ループ**に入る（アプリは全停止）。

スキーマは壊れていない（マイグレーションはトランザクション内で失敗しロールバックされる）。
復旧は「コンテナを止める → 手順2の INSERT → 起動」だけでよい。
本番 NAS では compose ファイル名が `docker-compose.yaml`、docker CLI は `sudo` が必要:

```bash
cd /volume1/docker/prompt_box
sudo docker compose -f docker-compose.yaml stop backend-rs
# …手順2の INSERT（"INSERT 0 1" が返れば成功）…
sudo docker compose -f docker-compose.yaml start backend-rs
```

## 手順

### 1. checksum を計算する

sqlx はファイル内容の **SHA-384** を `_sqlx_migrations.checksum` と突き合わせる。
リポジトリのファイルから計算すること（1バイトでも違えば起動時に VersionMismatch）。

```bash
shasum -a 384 backend-rs/migrations/20260711000000_initial_schema.sql | awk '{print $1}'
```

2026-07-25 時点の値:
`b4732a3cc0bb0c3833581cd5df8c2e5eb4e2ff325f1b0ff6dda5a993c1cd0519d696a35d2c17f6b4f1f7945c4490fa4f`

### 2. 適用済みとして記録する

```bash
CK=$(shasum -a 384 backend-rs/migrations/20260711000000_initial_schema.sql | awk '{print $1}')
docker compose exec -T db sh -c "psql -U \"\$POSTGRES_USER\" -d \"\$POSTGRES_DB\" -v ON_ERROR_STOP=1 \
  -c \"INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time) \
       VALUES (20260711000000, 'initial schema', NOW(), TRUE, decode('$CK','hex'), 0) \
       ON CONFLICT (version) DO NOTHING;\""
```

`description` は sqlx がファイル名から導出する値（`initial schema`）に合わせる。
検証されるのは version と checksum だけだが、`sqlx migrate info` の表示が揃う。

### 3. 確認してからデプロイする

```bash
# 2行（20260711000000 / 20260711000001）になっていること
docker compose exec -T db sh -c \
  'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT version, description FROM _sqlx_migrations ORDER BY version;"'
```

その後で新しい backend-rs イメージをデプロイする。起動ログに
`database migrations up to date` が出れば成功。

### 4. 切り戻し

記録行を消せば元に戻る（スキーマ自体は変更していない）。

```bash
docker compose exec -T db sh -c \
  'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "DELETE FROM _sqlx_migrations WHERE version = 20260711000000;"'
```

旧イメージに戻す場合は `db/init` のマウントを外したままでも問題ない（既存ボリュームでは初回起動時にしか
使われないため）。

## 検証済みの事実（2026-07-25、ローカル `comfyui_gallery` にて）

- 実 DB のスキーマと `20260711000000_initial_schema.sql` を空 DB に適用した結果を `pg_dump --schema-only` で
  突合し、**業務テーブル・インデックス・制約・トリガーが完全一致**することを確認
  （差分は `_sqlx_migrations` と Python 時代の遺物 `alembic_version` のみ）。
- 手動 INSERT 後に `sqlx migrate run` が no-op で成功することを確認。
- 空 DB に対して `sqlx migrate run` が全スキーマを再構成できることを確認。

## 以降のマイグレーション（通常運用）

B1 以降に追加されたマイグレーションは、backend-rs の起動時に自動適用される。手作業は不要。
ただし所要時間とロックには注意する。

| バージョン | 内容 | 本番での注意 |
|---|---|---|
| `20260725000001_fulltext_simple` | `images.search_vector`（STORED 生成列）追加 + GIN/trigram index、旧 english FTS index 削除 | **テーブル書き換え（ACCESS EXCLUSIVE）**。**実測 18.9 秒**（2026-07-25 本番・約42,000行。`slow statement` 警告が1本出るが正常）、その間 `images` への読み書きがブロックされる。index 作成は CONCURRENTLY ではない（マイグレーションはトランザクション内のため）。深夜など取り込みの少ない時間に再起動するのが無難 |
| `20260725000002_jobs` | `jobs` テーブル新設 | 新規テーブルのみ。既存データに触れないので一瞬 |
| `20260725000003_image_events` | `image_events` テーブル + `images` への変更フィードトリガー、**既存画像分の created イベントを backfill** | backfill は論理削除されていない画像1件につき1行。約42,000行の INSERT で数秒。以後 `images` の全 INSERT/UPDATE/DELETE でトリガーが1行書く（オーバーヘッドは軽微だが、取り込みバッチ中は `image_events` が増え続ける点に留意）。フィードの肥大が気になったら古い `seq` を削除してよい（下流が既に取得済みの範囲のみ） |

適用状況はいつでも次で確認できる:

```bash
docker compose exec -T db sh -c \
  'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT version, description, installed_on FROM _sqlx_migrations ORDER BY version;"'
```

## ⚠️ 恒久的な注意

**一度でも適用されたマイグレーションファイルは、コメント1行でも編集してはいけない。**
sqlx は SHA-384 を検証するため、内容を変えると既存 DB で `VersionMismatch` になり起動不能になる。
文言を直したくなったら、新しいマイグレーションファイルを追加するか、コード側のコメントに書く。
