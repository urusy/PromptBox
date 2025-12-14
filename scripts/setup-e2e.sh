#!/bin/bash
# E2Eテスト環境セットアップスクリプト

set -e

echo "=== Playwright MCP E2E Setup ==="
echo ""

# Node.jsバージョンチェック
NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo "❌ Node.js 18以上が必要です（現在: $(node -v)）"
    exit 1
fi
echo "✅ Node.js $(node -v)"

# Playwrightブラウザのインストール
echo ""
echo "📦 Playwrightブラウザをインストール中..."
npx playwright install

# Claude Code CLIの確認
if ! command -v claude &> /dev/null; then
    echo ""
    echo "⚠️  Claude Code CLIが見つかりません"
    echo "   インストール: npm install -g @anthropic-ai/claude-code"
    echo ""
    echo "Claude Code CLIをインストールしますか？ (y/n)"
    read -r INSTALL_CLAUDE
    if [ "$INSTALL_CLAUDE" = "y" ]; then
        npm install -g @anthropic-ai/claude-code
    fi
fi

# Playwright MCPの追加
echo ""
echo "🔧 Playwright MCPを設定中..."
if command -v claude &> /dev/null; then
    claude mcp add playwright -- npx @playwright/mcp@latest
    echo "✅ Playwright MCPを追加しました"
else
    echo "⚠️  Claude Code CLIがないため、手動で設定してください："
    echo ""
    echo "   claude mcp add playwright -- npx @playwright/mcp@latest"
fi

echo ""
echo "=== セットアップ完了 ==="
echo ""
echo "使い方:"
echo "  1. アプリケーションを起動: docker compose up -d"
echo "  2. Claude Codeを起動: claude"
echo "  3. テスト実行: 「playwright mcpを使用して http://localhost:3000 を開いて」"
echo ""
echo "詳細は docs/07_e2e_testing.md を参照してください。"
