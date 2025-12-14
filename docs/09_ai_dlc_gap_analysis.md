# AI-DLC ギャップ分析レポート

## AI-DLCとは

**AI-DLC（AI-Driven Development Life Cycle）** は、AWSが2025年7月に発表した次世代ソフトウェア開発方法論です。従来のSDLC（Software Development Life Cycle）をAIを中心に据えて再構築し、開発速度と品質を大幅に向上させることを目的としています。

### AI-DLCの3つのフェーズ

| フェーズ | 目的 | 主な活動 |
|---------|------|----------|
| **Inception（着想）** | WHAT/WHY | 要件分析、ユーザーストーリー作成、リスク評価、複雑性評価 |
| **Construction（構築）** | HOW | 詳細設計、コード生成、テスト戦略、品質保証 |
| **Operations（運用）** | RUN | デプロイメント自動化、監視・可観測性、本番環境検証 |

### AI-DLCの主要な特徴

1. **適応型ワークフロー**: プロジェクトの複雑性に応じて自動調整
2. **AI支援と人間の監視**: AIが計画・実装を担当し、人間が判断・承認
3. **Bolts（短期サイクル）**: 従来のSprintを時間・日単位に短縮
4. **Mob Elaboration/Construction**: チーム全体でのリアルタイム協業

---

## PromptBox現状分析

### 実装済み機能

| カテゴリ | 状況 | 詳細 |
|----------|------|------|
| 設計ドキュメント | ✅ 充実 | 要件定義、アーキテクチャ、API設計、DB設計など9文書 |
| バックエンド | ✅ 完成 | FastAPI + SQLAlchemy 非同期、3,282行 |
| フロントエンド | ✅ 完成 | React + TypeScript strict、5,702行 |
| 認証・セキュリティ | ✅ 実装 | bcrypt、JWT、Cookie セッション |
| メタデータパース | ✅ 実装 | ComfyUI/A1111/Forge/NovelAI対応 |
| Docker環境 | ✅ 完成 | dev/prod環境分離、PostgreSQL 16 |
| E2Eテスト | ✅ 環境構築 | Playwright MCP統合 |
| コードレビュー | ✅ 導入 | CodeRabbit自動レビュー |

---

## AI-DLC観点からのギャップ分析

### 1. Inceptionフェーズ（着想）

| AI-DLC要素 | PromptBox状況 | ギャップ | 優先度 |
|------------|---------------|----------|--------|
| 要件定義 | ✅ docs/01_requirements.md | - | - |
| ユーザーストーリー | ⚠️ 一部のみ | ストーリー形式での記述なし | 低 |
| リスク評価 | ⚠️ 部分対応 | バックアップ戦略に含む | 低 |
| 複雑性評価 | ❌ 未実施 | 機能別の複雑性マトリクスなし | 低 |
| ADR（Architecture Decision Records） | ✅ 対応済 | docs/adr/に5件作成 | - |

### 2. Constructionフェーズ（構築）

| AI-DLC要素 | PromptBox状況 | ギャップ | 優先度 |
|------------|---------------|----------|--------|
| 詳細設計 | ✅ 各種設計書 | - | - |
| コーディング規約 | ✅ CLAUDE.md | - | - |
| **ユニットテスト** | ✅ 対応済 | パーサー、サービス、ユーティリティのテスト実装 | - |
| **統合テスト** | ✅ 対応済 | API統合テスト実装 | - |
| テストカバレッジ | ✅ 対応済 | pytest-cov設定済み | - |
| **CI/CDパイプライン** | ✅ 対応済 | GitHub Actions設定完了 | - |
| 静的解析 | ✅ 対応済 | pyproject.toml（ruff, black, mypy）設定完了 | - |
| セキュリティスキャン | ✅ 対応済 | bandit, npm audit設定済み | - |
| 依存性脆弱性チェック | ✅ 対応済 | Dependabot設定完了 | - |

### 3. Operationsフェーズ（運用）

| AI-DLC要素 | PromptBox状況 | ギャップ | 優先度 |
|------------|---------------|----------|--------|
| デプロイ手順 | ✅ docs/07_deployment.md | - | - |
| **自動デプロイ** | ⚠️ 部分対応 | Dockerビルドまで自動化、本番デプロイは手動 | 低 |
| ログ集約 | ⚠️ 基本のみ | loguru使用、外部ログ基盤なし | 低 |
| **メトリクス監視** | ❌ 未導入 | Prometheus/Grafana等なし | 低 |
| **アラート設定** | ❌ 未設定 | エラー通知機構なし | 低 |
| **APM（Application Performance Monitoring）** | ❌ 未導入 | 応答時間・スループット計測なし | 低 |
| ヘルスチェック | ✅ 実装 | /api/health、/api/health/db | - |
| バックアップ戦略 | ✅ 対応済 | docs/10_backup_strategy.md作成 | - |
| 障害復旧計画 | ✅ 対応済 | バックアップ戦略にDR計画を含む | - |

---

## 優先度別 改善項目

### 🔴 高優先度（開発品質の基盤）

#### 1. ユニットテスト・統合テストの実装

```
backend/tests/
├── conftest.py              # pytest fixtures
├── unit/
│   ├── test_parsers/        # メタデータパーサーテスト
│   ├── test_services/       # サービス層テスト
│   └── test_utils/          # ユーティリティテスト
└── integration/
    ├── test_api/            # APIエンドポイントテスト
    └── test_db/             # データベース操作テスト
```

**必要な設定:**
- `pyproject.toml` - pytest、coverage設定
- `conftest.py` - テスト用DBセットアップ
- カバレッジ目標: 80%以上

#### 2. CI/CDパイプライン構築

```yaml
# .github/workflows/ci.yml
- name: Lint & Format Check
  # ruff, black, eslint, prettier

- name: Type Check
  # mypy, tsc

- name: Unit Tests
  # pytest with coverage

- name: Integration Tests
  # API tests with test DB

- name: Security Scan
  # bandit, npm audit

- name: Build Docker Images
  # Multi-stage build

- name: Deploy (on main merge)
  # 本番環境へのデプロイ
```

#### 3. 自動デプロイ（CD）

- Docker イメージのビルドとプッシュ
- 本番環境への自動デプロイ
- ロールバック機構

---

### 🟡 中優先度（品質向上）

#### 4. Python静的解析設定

```toml
# pyproject.toml
[tool.ruff]
line-length = 88
select = ["E", "F", "I", "N", "W", "UP", "B", "C4", "SIM"]

[tool.black]
line-length = 88

[tool.mypy]
python_version = "3.11"
strict = true

[tool.pytest.ini_options]
asyncio_mode = "auto"
testpaths = ["tests"]

[tool.coverage.run]
source = ["app"]
branch = true
```

#### 5. セキュリティスキャン導入

| ツール | 目的 |
|--------|------|
| bandit | Python静的セキュリティ分析 |
| safety | Python依存性脆弱性チェック |
| npm audit | Node.js依存性チェック |
| trivy | Dockerイメージ脆弱性スキャン |
| Dependabot | 依存性の自動更新PR |

#### 6. 監視・可観測性

```
推奨スタック:
- Prometheus: メトリクス収集
- Grafana: ダッシュボード・可視化
- Loki: ログ集約
- AlertManager: アラート通知
```

**計測すべきメトリクス:**
- リクエスト応答時間（P50, P95, P99）
- エラーレート
- DBコネクションプール使用率
- 画像取り込み処理時間
- ストレージ使用量

#### 7. リスク評価・ADR

```markdown
# docs/adr/
├── 0001-use-fastapi-async.md
├── 0002-uuid-v7-for-ids.md
├── 0003-metadata-parser-strategy.md
├── 0004-image-storage-structure.md
└── 0005-session-auth-over-jwt.md
```

#### 8. バックアップ戦略

- PostgreSQL: pg_dump定期実行、外部ストレージ保存
- 画像ストレージ: rsync/rcloneによる同期
- 保持期間: 日次7日、週次4週、月次12ヶ月

---

### 🟢 低優先度（将来的改善）

#### 9. ユーザーストーリー形式への変換

現在の要件定義を以下の形式に整理:
```
As a [ユーザー]
I want to [アクション]
So that [目的]

Acceptance Criteria:
- [ ] 条件1
- [ ] 条件2
```

#### 10. 障害復旧計画（DR）

- RTO（Recovery Time Objective）: 目標復旧時間
- RPO（Recovery Point Objective）: 目標復旧地点
- 手順書作成

#### 11. ログ集約基盤

- Loki + Promtail構成
- 構造化ログ（JSON形式）への移行
- ログ保持ポリシー

---

## 実装ロードマップ

### Phase 1: 基盤整備（推奨: 最優先）

1. `pyproject.toml` 作成（ruff, black, mypy, pytest設定）
2. `backend/tests/conftest.py` 作成
3. メタデータパーサーのユニットテスト実装
4. GitHub Actions CI設定（lint, type check, test）

### Phase 2: テスト充実

1. サービス層ユニットテスト
2. API統合テスト
3. カバレッジ計測・レポート
4. E2Eテストシナリオ実装

### Phase 3: セキュリティ・監視

1. セキュリティスキャン導入（bandit, safety, npm audit）
2. Dependabot設定
3. 基本的なメトリクス収集
4. アラート設定

### Phase 4: 運用改善

1. 自動デプロイ（CD）構築
2. バックアップ自動化
3. ADR作成
4. 監視ダッシュボード構築

---

## まとめ

### 対応完了項目 ✅

今回の対応により、以下の高優先度・中優先度項目が完了しました：

| 対応項目 | 状況 | 成果物 |
|---------|------|--------|
| Python静的解析設定 | ✅ 完了 | `backend/pyproject.toml` |
| ユニットテスト | ✅ 完了 | `backend/tests/unit/` (パーサー、サービス、ユーティリティ) |
| 統合テスト | ✅ 完了 | `backend/tests/integration/` (API テスト) |
| CI/CDパイプライン | ✅ 完了 | `.github/workflows/ci.yml` |
| セキュリティスキャン | ✅ 完了 | CI内でbandit, npm audit実行 |
| Dependabot | ✅ 完了 | `.github/dependabot.yml` |
| ADR | ✅ 完了 | `docs/adr/` (5件) |
| バックアップ戦略 | ✅ 完了 | `docs/10_backup_strategy.md` |

### 残存ギャップ（低優先度）

| ギャップ領域 | 影響 | 対応優先度 |
|-------------|------|-----------|
| 監視・可観測性 | 障害検知・パフォーマンス改善 | 🟢 低 |
| 完全自動デプロイ | 本番環境への自動反映 | 🟢 低 |
| ユーザーストーリー形式 | ドキュメント整備 | 🟢 低 |

これらの残存項目は、プロジェクトの成熟度に応じて段階的に対応することを推奨します。
AI-DLCの理念に沿った高品質な開発体制の基盤が整いました。

---

## 参考資料

- [AI-Driven Development Life Cycle: Reimagining Software Engineering | AWS DevOps Blog](https://aws.amazon.com/blogs/devops/ai-driven-development-life-cycle/)
- [Open-Sourcing Adaptive Workflows for AI-DLC | AWS DevOps Blog](https://aws.amazon.com/blogs/devops/open-sourcing-adaptive-workflows-for-ai-driven-development-life-cycle-ai-dlc/)
- [GitHub - awslabs/aidlc-workflows](https://github.com/awslabs/aidlc-workflows)
- [The AI-Driven Development Lifecycle: A critical, yet hopeful view | Medium](https://medium.com/data-science-collective/the-ai-driven-development-lifecycle-ai-dlc-a-critical-yet-hopeful-view-edc966173f2f)
- [AWS re:Invent 2025 - Introducing AI-DLC | DEV Community](https://dev.to/kazuya_dev/aws-reinvent-2025-introducing-ai-driven-development-lifecycle-ai-dlc-dvt214-32b)
