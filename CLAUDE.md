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

### バックエンド

- Python 3.11+
- FastAPI
- SQLAlchemy 2.0（非同期）
- asyncpg
- Pillow（画像処理）
- watchdog（フォルダ監視）
- uuid-utils（UUID v7）
- bcrypt
- cachetools（インメモリキャッシュ）

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

### Python（バックエンド）

- 型ヒントを必ず使用
- async/awaitを使用した非同期処理
- Pydanticでリクエスト/レスポンスのバリデーション
- ログはloguru推奨
- フォーマッタ: black
- リンター: ruff

```python
# 良い例
async def get_image(image_id: UUID) -> ImageResponse:
    ...

# 悪い例
def get_image(image_id):
    ...
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
├── backend/
│   ├── app/
│   │   ├── main.py          # エントリーポイント
│   │   ├── config.py        # 設定
│   │   ├── database.py      # DB接続
│   │   ├── api/             # APIエンドポイント
│   │   ├── services/        # ビジネスロジック
│   │   ├── repositories/    # データアクセス
│   │   ├── models/          # SQLAlchemyモデル
│   │   ├── schemas/         # Pydanticスキーマ
│   │   ├── parsers/         # メタデータパーサー
│   │   ├── workers/         # バックグラウンド処理
│   │   └── utils/           # ユーティリティ
│   └── tests/
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
├── db/
│   └── init/                # DB初期化SQL
│
├── import/                  # 画像取り込みフォルダ
└── storage/                 # 画像ストレージ
```

## 実装状況

### 完了済み

- Docker環境構築
- バックエンド基盤（FastAPI + DB接続）
- DBスキーマ
- 認証機能（Cookie セッション）
- メタデータパーサー（ComfyUI / A1111 / Forge / NovelAI）
- 画像取り込みワーカー（watchdog + 定期スキャン）
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

## 重要な実装ポイント

### UUID v7の使用

```python
from uuid_utils import uuid7

image_id = uuid7()
```

### 非同期DB操作

```python
from sqlalchemy.ext.asyncio import AsyncSession

async def get_image(db: AsyncSession, image_id: UUID) -> Image | None:
    result = await db.execute(select(Image).where(Image.id == image_id))
    return result.scalar_one_or_none()
```

### メタデータパース

```python
from app.parsers.factory import MetadataParserFactory

factory = MetadataParserFactory()
metadata = factory.parse(png_info)
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

```python
from cachetools import TTLCache

# タグ・統計データ用のインメモリキャッシュ
cache = TTLCache(maxsize=100, ttl=300)  # 5分
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

- バックエンド: pytest + pytest-asyncio
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

```bash
docker compose exec backend alembic upgrade head
```

### フロントエンドビルドエラー

```bash
docker compose exec frontend npm install
```

### 画像が表示されない

- storageディレクトリのパーミッション確認
- Nginxの設定確認（/storage/のalias）
