# CodeRabbit導入ガイド

## 概要

CodeRabbitはAIを活用したコードレビューツールで、Claude Codeとの公式統合を持っています。
この連携により、以下のワークフローが可能になります：

- Claude Codeで機能実装 → CodeRabbit CLIでレビュー → 問題を自動修正
- `--prompt-only`フラグでトークン効率的なレビュー
- `CLAUDE.md`を自動的に読み込んでコーディング標準を適用

## セットアップ手順

### 1. GitHubリポジトリをPublicに変更

1. GitHub上でリポジトリ設定を開く
2. 「Danger Zone」→「Change visibility」でPublicに変更

> **注意**: 無料プランはPublicリポジトリのみ対応しています

### 2. CodeRabbit CLIのインストール

```bash
curl -fsSL https://cli.coderabbit.ai/install.sh | sh
source ~/.zshrc
```

### 3. 認証

Claude Code内で以下を実行：

```bash
coderabbit auth login
```

1. 表示されたURLをブラウザで開く
2. GitHubアカウントで認証
3. 取得したトークンをターミナルに貼り付け

認証は複数のClaude Codeインスタンス間で永続化されます。

### 4. GitHub App連携（オプション）

PRベースの自動レビューを有効にする場合：

1. [CodeRabbit GitHub App](https://github.com/apps/coderabbitai)をインストール
2. 対象リポジトリを選択

## Claude Codeとの連携方法

### 方法A: CLIベース（推奨）

コード変更後、以下のコマンドを実行：

```bash
coderabbit --prompt-only
```

これにより：
- 変更内容を分析
- 問題点をリスト化
- ファイル位置と行番号を含むフィードバック

`--prompt-only`フラグはトークン効率的で、AIエージェント統合に最適化されています。

### 方法B: PRベース

1. 変更をブランチにプッシュ
2. PRを作成
3. CodeRabbitが自動でPRにレビューコメント
4. Claude Codeでコメントを確認・修正

## 典型的なワークフロー

```
ユーザー: 「新機能Xを実装して」

Claude Code:
1. 機能を実装
2. `coderabbit --prompt-only` を実行
3. 検出された問題を修正
4. 再度レビュー実行（問題がなくなるまで）
5. コミット・プッシュ
```

指示例：
> 「機能を実装後、`coderabbit --prompt-only`をバックグラウンド実行して、すべての問題を修正してください」

## 設定ファイル

`.coderabbit.yaml`をリポジトリルートに配置してカスタマイズ可能：

```yaml
reviews:
  auto_review:
    enabled: true
  path_filters:
    - "!**/*.md"  # マークダウンは除外
```

## トラブルシューティング

### 問題が検出されない場合

- `--base develop`で比較ブランチを指定
- `--type uncommitted`でスコープを限定

### 認証エラーの場合

```bash
coderabbit auth logout
coderabbit auth login
```

## 注意事項

- **Public必須**: 無料プランはPublicリポジトリのみ対応
- **CLAUDE.md活用**: 既存の`CLAUDE.md`をCodeRabbitが参照してコーディング標準を適用
- **トークン効率**: `--prompt-only`フラグで出力を簡潔に

## 参考リンク

- [CodeRabbit Claude Code Integration](https://docs.coderabbit.ai/cli/claude-code-integration)
- [CodeRabbit CLI](https://www.coderabbit.ai/cli)
- [CodeRabbit GitHub App](https://github.com/apps/coderabbitai)
