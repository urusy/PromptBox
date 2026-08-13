# CLAUDE.md - Claude Code向けプロジェクト指示

## 重要なルール

- **すべてのコミュニケーションは日本語で行うこと**
- 指示があるまでコミットとプッシュは勝手に行わないこと

## プロジェクト概要

ComfyUIやStable Diffusionで生成した画像を管理するウェブアプリケーション（Prompt Box）。
自動取り込み、メタデータ抽出、検索、評価、タグ付けなどの機能を提供する。

## ドキュメント

設計ドキュメントは `docs/` ディレクトリに格納されている。実装前に必ず参照すること。

- `docs/01_requirements.md` - 要件定義
- `docs/02_architecture.md` - 技術スタック・アーキテクチャ
- `docs/03_database_schema.md` - データベーススキーマ
- `docs/04_api_design.md` - API設計
- `docs/05_metadata_parser.md` - メタデータパーサー設計
- `docs/06_docker_infrastructure.md` - Docker構成

## 技術スタック

### バックエンド（`backend-rs`）

- Rust（edition 2024）
- axum 0.8 + tower / tower-http（ミドルウェア）
- sqlx 0.8（PostgreSQL・マイグレーション・統合テスト）
- tokio（非同期ランタイム・ジョブ実行）
- image / png / webp / kamadak-exif（画像処理・メタデータ）
- notify（フォルダ監視）
- object_store（MinIO / S3 互換ストレージ）
- uuid v7 / bcrypt / jsonwebtoken
- 自前の TTL キャッシュ（`src/cache.rs`）

> Python(FastAPI) 版は 2026-07-25 に撤去済み。設計ドキュメントの Python 記述は歴史的経緯として残る。

### フロントエンド

- React 18
- TypeScript 5+
- TailwindCSS 3
- TanStack Query 5
- TanStack Virtual（仮想スクロール）
- React Router 6
- Axios
- Zustand
- Recharts（統計グラフ）

### インフラ

- Docker / Docker Compose
- PostgreSQL 16
- Nginx

## コーディング規約

### Rust（バックエンド）

- エラーは `AppError` に集約し、`?` で伝播する（ハンドラで `unwrap`/`expect` を使わない）
- SQL の値は必ず `push_bind` / `$1` でバインドする（文字列連結は SQL インジェクション）
- リクエスト/レスポンスの型は `src/dto/` に置き、serde で定義する
- ログは `tracing`（構造化フィールドで出す。`tracing::info!(count = n, "...")`）
- リンター: `cargo clippy --all-targets -- -D warnings`（フォーマッタは既存差分のため未適用）

```rust
// 良い例（コンパイル時DB接続が要る query_as! マクロは使わない。ビルドコンテナにDBが無いため）
pub async fn get_image(pool: &PgPool, id: Uuid) -> Result<Option<ImageRow>, sqlx::Error> {
    sqlx::query_as::<_, ImageRow>("SELECT * FROM images WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

// 悪い例（文字列連結・panic）
let sql = format!("SELECT ... WHERE id = '{id}'");
let row = sqlx::query(&sql).fetch_one(pool).await.unwrap();
```

### TypeScript（フロントエンド）

- 厳格な型定義（`strict: true`）
- React Hooks使用
- 関数コンポーネントのみ
- フォーマッタ: prettier
- リンター: eslint

```typescript
// 良い例
interface ImageCardProps {
  image: Image;
  onSelect: (id: string) => void;
}

const ImageCard: React.FC<ImageCardProps> = ({ image, onSelect }) => {
  ...
};

// 悪い例
const ImageCard = (props: any) => {
  ...
};
```

### CSS（TailwindCSS）

- ユーティリティクラス優先
- カスタムCSSは最小限
- レスポンシブはモバイルファースト

```tsx
// 良い例
<div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">

// 悪い例
<div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)' }}>
```

### DBマイグレーション

マイグレーションは、本番環境があることを前提として、バージョンを分けてマイグレーションを行えるようにすること。

## ディレクトリ構成

```text
prompt-box/
├── docker-compose.yml
├── .env.example
├── CLAUDE.md                 # このファイル
├── docs/                     # 設計ドキュメント
│
├── backend-rs/
│   ├── build.rs             # GIT_SHA / ビルド時刻の埋め込み
│   ├── migrations/          # sqlx マイグレーション（スキーマの唯一の真実）
│   ├── src/
│   │   ├── main.rs          # エントリーポイント（薄い）
│   │   ├── lib.rs           # ライブラリクレート promptbox
│   │   ├── config.rs        # 設定
│   │   ├── db.rs            # 接続プール
│   │   ├── http/            # ルータ・ハンドラ（ミドルウェアもここ）
│   │   ├── dto/             # リクエスト/レスポンス型
│   │   ├── image/           # 検索・更新・近傍などのSQLロジック
│   │   ├── parser/          # メタデータパーサー
│   │   ├── worker/          # 取り込みワーカー
│   │   ├── job/             # 非同期ジョブ基盤
│   │   ├── change/          # 変更フィード
│   │   └── storage.rs       # MinIO/S3 アクセス
│   └── tests/               # #[sqlx::test] による統合テスト
│
├── frontend/
│   └── src/
│       ├── api/             # API通信
│       ├── components/      # UIコンポーネント
│       ├── pages/           # ページ
│       ├── hooks/           # カスタムフック
│       ├── stores/          # 状態管理
│       ├── types/           # 型定義
│       └── utils/           # ユーティリティ
│
├── import/                  # 画像取り込みフォルダ
└── minio-data/              # MinIO 専有（ホストから直接触らない）
```

## 実装状況

### 完了済み

- Docker環境構築
- バックエンド基盤（Rust/axum + sqlx。Python/FastAPI 版から全面移行し 2026-07-25 に撤去）
- DBスキーマ（sqlx マイグレーションに一本化）
- 認証機能（Cookie セッション）
- メタデータパーサー（ComfyUI / A1111 / Forge / NovelAI）
- 画像取り込みワーカー（notify + 定期スキャン）
- 画像API（CRUD + 一括操作）
- フロントエンド（React + TailwindCSS）
- 一覧画面（グリッド表示、ページネーション、仮想スクロール）
- 詳細画面（prev/next ナビゲーション）
- 検索機能（プリセット、スマートフォルダ）
- 評価・タグ機能
- 一括操作（評価、タグ、削除）
- エクスポート機能
- ゴミ箱機能
- 重複検出
- 統計ページ（Recharts）
- パフォーマンス最適化（キャッシュ、Code Splitting、仮想スクロール）
- Showcase機能（画像コレクション、スライドショー、カバー画像選択）
- 検索フィルター拡張（LoRA、Sampler、日付範囲、未評価のみ、タグ複数AND条件、Orientation）
- Model/LoRA一覧・詳細ページ（CivitAI連携、推奨設定表示）
- Quick Rate機能（Tinder風スワイプUI、キーボードショートカット）
- E2Eテスト環境（Playwright MCP）
- iOS Safari対応（モーダルのbody scroll lock、Escキー対応、カスタム確認ダイアログ）
- CivitAI検索改善（モデル名正規化、あいまいマッチング改善）
- CivitAI情報のバージョンタブ表示（モデル/LoRA詳細ページ）
- モデル一覧のバージョングループ化（`_v80`等のサフィックス除去、統計集計）
- Seed検索機能（±300のトレランス範囲検索、Model+Seed複合検索）
- レスポンシブUI改善（iPad Pro対応、ナビゲーションのxlブレークポイント最適化）
- 検索フォームUI改善（プリセット・検索ボックスの横並び配置、ボタンサイズ統一）

## 重要な実装ポイント

### UUID v7の使用

```rust
use uuid::Uuid;

let image_id = Uuid::now_v7();
```

### 動的な検索クエリ

```rust
// 条件が可変なクエリは QueryBuilder。値は必ず push_bind でバインドする
let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM images WHERE deleted_at IS NULL");
if let Some(v) = &p.model_name {
    qb.push(" AND model_name = ").push_bind(v);
}
let items = qb.build_query_as::<ImageRow>().fetch_all(pool).await?;
```

### メタデータパース

```rust
use promptbox::parser;

// png_info: HashMap<String, String>（PNG テキストチャンク）
let metadata = parser::parse(&png_info);
```

### レスポンシブ対応

```tsx
// TailwindCSSのブレークポイント
// sm: 640px, md: 768px, lg: 1024px, xl: 1280px, 2xl: 1536px

<div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
```

### Safari対応

```css
/* 100vh問題の回避 */
min-height: 100dvh;

/* Safe Area対応 */
padding-bottom: env(safe-area-inset-bottom);
```

### キャッシュ（バックエンド）

```rust
use promptbox::cache::TtlCache;

// タグ・統計データ用のインメモリキャッシュ（src/cache.rs）
let cache: TtlCache<Value> = TtlCache::new(Duration::from_secs(300), 100); // 5分
```

### Code Splitting（フロントエンド）

```tsx
// 使用頻度の低いページは遅延ロード
const StatsPage = lazy(() => import('@/pages/StatsPage'))

<Suspense fallback={<PageLoader />}>
  <Route path="stats" element={<StatsPage />} />
</Suspense>
```

### 仮想スクロール（フロントエンド）

```tsx
// 100件以上の画像表示時に自動適用
import { useVirtualizer } from '@tanstack/react-virtual'
```

## テスト方針

- バックエンド（`backend-rs` / Rust）: `cargo test`
  - ユニットテスト: 各モジュールの `#[cfg(test)] mod tests`（パーサ・純粋関数・バリデーション）
  - 統合テスト: `backend-rs/tests/` に `#[sqlx::test(migrations = "./migrations")]` で書く。
    テストごとに使い捨ての DB が自動作成され、`backend-rs/migrations/` が適用される。
    実行にはホストから DB に届く `DATABASE_URL` が必要:

    ```bash
    docker compose up -d db   # ${DB_PORT:-5433} で公開される
    export DATABASE_URL="postgres://<DB_USER>:<DB_PASSWORD>@localhost:5433/<DB_NAME>"
    cd backend-rs && cargo test
    ```

    SQL を含むロジック（検索フィルタ・prev/next・一括操作）は必ずここで検証する。
- フロントエンド: Vitest + React Testing Library
- E2E: 不要（1ユーザーアプリのため）

## コードレビュー方針

- Code Rabbitを設定しているため `coderabbit --prompt-only` のコマンドを使用してレビューを受けること
- 上記で指摘があれば対応し、再度コマンドを実行してレビューを受けること
- 指摘がなくなるまで繰り返すこと

## 注意事項

1. **画像ファイルの取り扱い**
   - 元ファイルは必ず保持
   - サムネイルはWebP形式で生成
   - ファイルパスはUUID v7ベースで分散配置

2. **メタデータパース**
   - パースエラーは吸収し、取得できた情報のみ保存
   - 未対応フォーマットはhas_metadata=falseで登録

3. **削除の二段階**
   - 最初は論理削除（deleted_atに日時設定）
   - ゴミ箱から完全削除で物理削除

4. **セキュリティ**
   - パスワードはbcryptでハッシュ化
   - セッションCookieはHttpOnly, Secure, SameSite=Strict
   - ファイルパスのトラバーサル対策

## トラブルシューティング

### DBマイグレーションエラー

マイグレーションは backend-rs の起動時に `sqlx::migrate!` が自動適用する。手動実行は不要。
適用状況の確認と、既存DBへベースラインを記録する手順は
`docs/runbooks/schema-baseline-migration.md` を参照（**適用済みファイルの編集は厳禁**）。

```bash
docker compose exec -T db sh -c \
  'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT version, description FROM _sqlx_migrations ORDER BY version;"'
```

### フロントエンドビルドエラー

```bash
docker compose exec frontend npm install
```

### 画像が表示されない

- 画像は MinIO（オブジェクトストレージ）にあり、backend-rs の `/storage/{path}` が配信する
- MinIO コンテナの起動と `minio-init`（バケット作成）の完了を確認
- Nginxの設定確認（/storage/ を backend-rs へプロキシしているか）
