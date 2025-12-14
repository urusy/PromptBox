# Architecture Decision Records (ADR)

このディレクトリには、Prompt Boxプロジェクトの重要な設計判断を記録したADR（Architecture Decision Records）が含まれています。

## ADRとは

ADRは、ソフトウェアアーキテクチャに関する重要な決定を文書化するための形式です。各ADRには以下が含まれます：

- **コンテキスト**: なぜこの決定が必要だったか
- **決定**: 何を決定したか
- **結果**: この決定によってもたらされる影響

## ADR一覧

| ID | タイトル | ステータス | 日付 |
|----|---------|----------|------|
| [0001](0001-use-fastapi-async.md) | FastAPIと非同期処理の採用 | 採用 | 2024-01 |
| [0002](0002-uuid-v7-for-ids.md) | UUID v7をプライマリキーとして使用 | 採用 | 2024-01 |
| [0003](0003-metadata-parser-strategy.md) | メタデータパーサーのストラテジーパターン | 採用 | 2024-01 |
| [0004](0004-image-storage-structure.md) | 画像ストレージの分散配置構造 | 採用 | 2024-01 |
| [0005](0005-session-auth.md) | Cookieベースセッション認証の採用 | 採用 | 2024-01 |

## ADRテンプレート

新しいADRを作成する際は、以下のテンプレートを使用してください：

```markdown
# ADR-XXXX: タイトル

## ステータス

提案 / 採用 / 非推奨 / 置き換え

## コンテキスト

この決定が必要になった背景や状況を説明します。

## 決定

何を決定したかを明確に記述します。

## 結果

この決定によってもたらされる影響（良い面・悪い面の両方）を記述します。

## 代替案

検討した他の選択肢とそれを選ばなかった理由を記述します。
```

## 参考リンク

- [ADR GitHub Organization](https://adr.github.io/)
- [Documenting Architecture Decisions - Michael Nygard](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
