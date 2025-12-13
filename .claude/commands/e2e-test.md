# E2Eテスト実行コマンド

Playwright MCPを使用してE2Eテストを実行します。

## 実行手順

1. アプリケーションが起動していることを確認（http://localhost:3000）
2. Playwright MCPでブラウザを開く
3. 指定されたテストシナリオを実行

## テストシナリオ

$ARGUMENTS が指定された場合、そのシナリオを実行：

### 基本機能
- `auth` - 認証フロー（ログイン/ログアウト）
- `gallery` - 画像ギャラリー（一覧、検索、フィルター）
- `detail` - 画像詳細（評価、タグ、ナビゲーション）
- `swipe` - Quick Rate（キーボード、ボタン操作）
- `models` - Models/LoRAs（一覧、詳細、CivitAI）
- `showcase` - Showcase（作成、編集、D&D）
- `mobile` - モバイル対応（レスポンシブ、タッチ）

### 管理機能
- `trash` - ゴミ箱（復元、完全削除）
- `duplicates` - 重複検出（一覧、解決）
- `smart-folders` - スマートフォルダ（CRUD）
- `stats` - 統計ページ（グラフ、ランキング）
- `presets` - 検索プリセット（保存、適用）
- `batch` - 一括操作（評価、タグ、削除）
- `export` - エクスポート（単一、一括）

### フィルターテスト
- `filters` - 全フィルター（Orientation、日付、Sampler等）

### 全体
- `all` - 全テスト実行

指定がない場合は `all` を実行。

## 実行

Playwright MCPを使用して以下のテストを実行してください：

1. http://localhost:3000 を開く
2. docs/07_e2e_testing.md のテストシナリオに従ってテストを実行
3. 各ステップの結果を報告
4. 問題があれば詳細を報告
