# E2Eテスト実行コマンド

Playwright MCPを使用してE2Eテストを実行します。

## 実行手順

1. アプリケーションが起動していることを確認（http://localhost:3000）
2. Playwright MCPでブラウザを開く
3. 指定されたテストシナリオを実行

## テストシナリオ

$ARGUMENTS が指定された場合、そのシナリオを実行：
- `auth` - 認証フロー（ログイン/ログアウト）
- `gallery` - 画像ギャラリー（一覧、検索、フィルター）
- `detail` - 画像詳細（評価、タグ、ナビゲーション）
- `swipe` - Quick Rate（キーボード、ボタン操作）
- `models` - Models/LoRAs（一覧、詳細、CivitAI）
- `showcase` - Showcase（作成、編集、D&D）
- `all` - 全テスト実行

指定がない場合は `all` を実行。

## 実行

Playwright MCPを使用して以下のテストを実行してください：

1. http://localhost:3000 を開く
2. docs/07_e2e_testing.md のテストシナリオに従ってテストを実行
3. 各ステップの結果を報告
4. 問題があれば詳細を報告
