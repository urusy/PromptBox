# Falcon 統合とデータ移行の検討

作成日: 2026-07-06 / ステータス: 検討完了（方針決定待ち）

PromptBox と Falcon（`../Falcon`）の統合可否・データ移行方法を検討し、方針の選択肢と推奨案をまとめる。

## 結論（サマリ）

- **推奨: 案C「役割分担による段階的統合」**。PromptBox を「AI生成画像の取り込み・メタデータエンジン」、Falcon を「長期アーカイブ・DAM」と位置づけて併存させ、既存の Falcon 側 PromptBox 連携（bulk-import）でデータを Falcon へ流し込めるパイプを維持する。
- **完全吸収（案A）は現時点では見送り**。Falcon は AI メタデータの**パーサーを持たない**（`ai_image_metadata` は API 経由でしか作られない）ため、吸収にはパーサー・CivitAI 連携・統計・Quick Rate 等の大規模移植が必要になる。
- データ移行自体は **Falcon 側に実装済みの `POST /api/v1/promptbox/bulk-import` でいつでも実行可能**。スキーマ対応付けも実装済みでギャップが小さい（後述のマッピング表）。ロックインリスクは低い。
- 再検討タイミング: Svelte フロント再構築に着手するかどうかを判断するとき。フロントを作り直すくらいなら「UI を Falcon に寄せ、PromptBox を headless 化する」案（C-2）が有力になる。

## 1. 背景

- PromptBox は Rust(axum) + Svelte で再構築中（`backend-rs/` は全14系統 API・パーサー・取り込みワーカーまで移植完了、Svelte フロントは未着手）。
- Falcon は Go(Gin/GORM) + React 19 の汎用 DAM で、機能的には PromptBox のスーパーセットに見える（フォルダ階層・タググループ・コレクション・多様なインポート経路・マルチユーザー対応スキーマ）。
- 2つの類似アプリを維持するコストを下げたい一方、PromptBox には生成ワークフロー特化の独自価値がある。

## 2. 現状比較

| 観点 | PromptBox | Falcon |
|---|---|---|
| バックエンド | FastAPI(:8000) + backend-rs(:8001) 並走 | Go 1.23 + Gin + GORM(:8080) |
| フロント | React 18（Svelte 再構築は未着手） | React 19 + Zustand |
| DB | PostgreSQL 16（`comfyui_gallery`、外部共有コンテナ `promptbox-db-1`） | PostgreSQL 16（falcon-db） + Redis |
| ユーザーモデル | シングルユーザー（Cookie セッション/JWT HS256） | スキーマはマルチユーザー対応（JWT + API Key + users テーブル） |
| 画像データ | `images` 1テーブルに AI メタデータをインライン保持 | `files` + `ai_image_metadata`（1:1）に分離 |
| タグ | `images.user_tags` JSONB 文字列配列 | `tags` / `file_tags` / `tag_groups` の正規化テーブル |
| ストレージ | `<hash[:2]>/<hash[2:4]>/<sha256>.<ext>`、サムネ `thumbnails/../<hash>.webp`（WebP q85） | `originals/l1/l2/<uniqueID>.<ext>`、サムネ `thumbnails/../<id>_thumb.<ext>` |
| 重複検出 | SHA-256（取り込み時に import/duplicated/ 退避） | SHA-256 + perceptual hash + source_url |
| AI メタデータ抽出 | **自前パーサー**（ComfyUI / A1111 / Forge / NovelAI、Python・Rust 両実装） | **なし**（API 経由の登録のみ） |
| 動画 | 非対応 | 対応（サムネ・タイムライン注釈） |

### AI メタデータスキーマの対応度

Falcon の `ai_image_metadata`（`gateway/internal/model/models.go`）は PromptBox の `images` の AI メタ列を**ほぼ意図的にミラー**している:

source_tool / model_type / has_metadata / positive_prompt / negative_prompt / model_name / sampler_name / scheduler / steps / cfg_scale / seed / loras(JSONB) / controlnets(JSONB) / embeddings(JSONB) / model_params(JSONB) / workflow_extras(JSONB) / raw_metadata(JSONB) / needs_improvement — **全て一致**。

→ メタデータの受け皿としてのスキーマギャップは実質ゼロ。

## 3. 既存の連携資産（実装済み）

### Falcon 側（`gateway/internal/{infrastructure,application,handler}/promptbox/`）

- Cookie 認証クライアント（`PROMPTBOX_BASE_URL/USERNAME/PASSWORD` で設定）
- `POST /api/v1/promptbox/images/:id/save` — 単一取込（画像ダウンロード + メタデータ + user_tags→tags 変換 + rating/needs_improvement 引き継ぎ）
- `POST /api/v1/promptbox/bulk-import` — フィルタ指定の一括取込（バックグラウンド、進捗 API、最大100ページ）
- 重複判定: `files.source_url`（= PromptBox の `original_url`）

### PromptBox 側（backend-rs `src/dto/image.rs`）

- Falcon が期待する形式（`original_url` / `thumbnail_url` / nested `pagination` / `user_tags` / `needs_improvement`）と現フロント形式（`storage_path` / フラット pagination）の**スーパーセット出力**を実装済み。ドリフト修復済み。

## 4. データ移行マッピング

### images → files + ai_image_metadata

| PromptBox `images` | Falcon | 変換 |
|---|---|---|
| id (UUID v7) | — | Falcon 側で新規 ID 採番。対応は `files.source_url` に PromptBox URL として残る |
| storage_path / file_hash | `files.storage_path` / `file_hash` | bulk-import が再ダウンロード→再ハッシュ→Falcon 規約で再配置（パス規約差は自然に吸収） |
| original_filename, width, height, file_size_bytes | `files.*` | そのまま |
| rating (0-5) | `files.rating` | そのまま（クランプ済み） |
| is_favorite | `files.is_favorite` | そのまま |
| user_memo | `files.memo` | そのまま |
| user_tags (JSONB配列) | `tags` + `file_tags` | 取込時に自動タグ作成（実装済み）。タググループは未分類になる |
| needs_improvement | `ai_image_metadata.needs_improvement` | そのまま |
| AI メタ一式（prompt〜raw_metadata） | `ai_image_metadata.*` | 1:1 対応（実装済み） |
| created_at | `files.file_created_at` | 要確認: bulk-import が生成日時を引き継ぐか（取込日時になると時系列が壊れる） |
| deleted_at（論理削除分） | — | **対象外**。移行前にゴミ箱を空にするか、削除分は捨てる判断 |

### その他エンティティ

| PromptBox | Falcon | 移行性 |
|---|---|---|
| showcases / showcase_images（順序・カバー付き） | collections（cover_file_id あり） | **自動移行の仕組みなし**。件数少なければ手動再構築、多ければ小スクリプト（両 API を叩く）で対応可 |
| smart_folders（filters JSONB） | smart_folders + smart_folder_conditions（field/operator/value） | フィルタ表現が異なる（JSONB 条件 vs 正規化条件行）。**AI メタ系フィールド（model_name, lora 等）が Falcon の condition field にあるか要確認**。なければ移行不能な条件が残る |
| search_presets | フィルタープリセット相当（クイックアクセス） | 同上。実質は手動再作成が現実的 |
| import/duplicated/ 内の退避ファイル | — | 移行対象外（未取込ファイルのため） |

### ストレージ移行方式の選択

1. **API 経由（推奨）**: bulk-import に任せる。パス規約・サムネイル形式の差を意識する必要がなく、DB とファイルの整合が Falcon 側ロジックで保証される。欠点は HTTP 経由のため全量で時間がかかること（数万枚規模なら夜間バッチで現実的）。
2. 直接コピー + SQL 変換: 高速だが、パス規約変換・ID 採番・サムネ再生成を自前実装することになり、bulk-import があるのに車輪の再発明。**不採用**。

## 5. ギャップ分析（Falcon に無い PromptBox 固有機能)

| 機能 | Falcon 側の状況 | 吸収時の移植コスト |
|---|---|---|
| **メタデータパーサー**（ComfyUI ノードグラフ解決 / A1111 / Forge / NovelAI） | **なし**。Watch Folder は AI メタを抽出しない | 大（Go への移植 or PromptBox パーサーのマイクロサービス化） |
| **取り込みワーカー**（ComfyUI 出力フォルダ監視 → パース → 登録 → 失敗隔離） | Watch Folder はあるがメタ抽出なし | 中〜大（パーサー移植とセット） |
| CivitAI 連携（モデル/LoRA 詳細、名称正規化、ハッシュ照会、推奨設定） | なし | 中 |
| 統計・評価傾向分析（rating-analysis、モデル×評価分布等） | 統計はストレージ/ファイル数中心。AI メタ横断の分析なし | 中 |
| Quick Rate（スワイプ評価 UI） | なし | 小〜中 |
| Gelbooru タグ検索 | なし（Sankaku 等はある） | 小 |
| モデル/LoRA カタログページ（バージョングルーピング） | なし | 中 |
| Seed 検索（±トレランス）、prev/next ナビ 等の検索特化 UI | 汎用検索のみ | 中 |

**評価**: 「画像の入口（生成→取込→パース）」と「生成品質のフィードバックループ（評価・分析・CivitAI）」が PromptBox の固有価値。ここを Falcon に移すコストが統合の支配項。

## 6. 統合方針の選択肢

### 案A: Falcon へ完全吸収（PromptBox 廃止）

bulk-import で全量移行し、パーサー・ワーカー・CivitAI・統計・Quick Rate を Falcon(Go) に移植。

- 長所: 運用が1アプリに集約。タグ正規化・フォルダ階層・動画などFalconの資産を全部使える。
- 短所: **移植工数が大**（特にパーサー。ComfyUI ノード走査の忠実移植は backend-rs で一度やったばかり）。完成した backend-rs の大半が無駄になる。移植完了まで生成ワークフローが劣化。
- 工数目安: 大（数週間〜）

### 案B: 併存・連携強化（現状維持+）

両アプリを独立運用し、選抜画像（高評価等）だけ bulk-import で Falcon にアーカイブする運用を確立。

- 長所: 追加開発ほぼゼロ。すぐ始められる。各アプリの強みをそのまま使える。
- 短所: 2アプリ分の運用継続。Svelte フロント再構築という宿題が残ったまま。評価等のメタデータが取込後に二重管理になる（Falcon 取込時点のスナップショット）。
- 工数目安: 小（運用手順の整備のみ）

### 案C: 役割分担による段階的統合（推奨）

案Bから始め、**PromptBox を「取り込み・メタデータエンジン」に純化**していく。

- フェーズ1（すぐ）: 案Bと同じ。bulk-import パイプの疎通確認と、選抜 or 全量アーカイブ運用の開始。
- フェーズ2（Svelte 着手判断時）: **Svelte フロントは作らない**判断をした場合、閲覧・整理 UI を Falcon に寄せる（C-2）。PromptBox は backend-rs の worker+parser+API だけを headless 運用し、Falcon が定期 bulk-import（または save API）で吸い上げる。
- フェーズ3（任意・将来）: Quick Rate / 統計 / CivitAI のうち手放せないものだけ Falcon に順次移植し、完了したら PromptBox の UI・重複機能を撤去。最終形は案Aに漸近。
- 長所: 各段階で止まれる・戻れる。backend-rs の投資（パーサー・ワーカー）が最終形でも活きる。フロント二重開発（Svelte）を回避できる可能性。
- 短所: 移行期間中は2アプリ併存が続く。評価の二重管理問題はフェーズ2で「評価は Falcon 側で行う」と決めるまで残る。
- 工数目安: フェーズ1=小、フェーズ2=中（Falcon 側に増分取込の定期実行・PromptBox 側の headless 化）、フェーズ3=中〜大（対象機能による）

### 比較まとめ

| | 案A 吸収 | 案B 併存 | 案C 段階統合 |
|---|---|---|---|
| 初期工数 | 大 | 小 | 小 |
| 最終運用コスト | 最小（1アプリ） | 2アプリ継続 | フェーズ次第で1アプリに収束 |
| backend-rs 投資の活用 | ほぼ無駄 | 全部活きる | パーサー/ワーカーが活きる |
| Svelte フロント問題 | 解消（不要になる） | 未解決 | フェーズ2で解消可 |
| ロールバック | 困難 | 不要 | 各フェーズで可能 |

## 7. 移行手順（案C フェーズ1 / 案A 共通の初回一括取込）

1. **事前準備**
   - PromptBox のゴミ箱を空にする（論理削除分を移行対象から確定的に除外）
   - `import/duplicated/`・`import/failed/` を整理
   - Falcon の `PROMPTBOX_BASE_URL/USERNAME/PASSWORD` を設定（NAS 上はコンテナ間ネットワーク越し。`/api/v1/promptbox/status` で疎通確認）
   - Falcon 側のディスク空き容量確認（PromptBox storage/ と同量+サムネ分）
2. **小規模試行**: フィルタを絞って bulk-import（例: `min_rating=5` の数十枚）。以下を検証:
   - AI メタデータ（prompt/loras/model_params）が `ai_image_metadata` に完全転記されているか
   - user_tags がタグとして作成されているか / rating・needs_improvement の引き継ぎ
   - **created_at（生成日時）の扱い** — 取込日時で潰れる場合は Falcon 側改修 or 許容の判断
   - 画像・サムネイルの表示品質
3. **本移行**: フィルタなし bulk-import（進捗は `/api/v1/promptbox/bulk-import/progress`）。100ページ上限があるため件数によっては date_from 等で分割実行
4. **検証**: 件数突合（PromptBox の総数 − 論理削除 = Falcon の source=promptbox 件数）、抜けは `/api/v1/promptbox/saved-ids` と突合
5. **ロールバック**: Falcon 側で `files.source = 'promptbox'`（相当のソース識別）を一括削除するだけ。**PromptBox 側は無変更なのでリスクなし**

## 8. 未確認事項（次アクション）

- [ ] bulk-import が `created_at`（生成日時）を `file_created_at` に引き継ぐか（コード確認 or 試行で判定）
- [ ] Falcon の smart_folder_conditions の field に AI メタ系（model_name / lora / sampler）があるか
- [ ] bulk-import の100ページ上限 × per_page で全量をカバーできるか（現在の総画像数の確認）
- [ ] 本番 NAS 上で Falcon と PromptBox のコンテナネットワークが疎通するか（別 compose・別 network の場合 external network 接続が必要）
- [ ] 評価の二重管理ポリシー: Falcon 取込後に PromptBox 側で評価を変えた場合の再同期は現状ない（source_url 重複でスキップされる）。「取込後の評価は Falcon で行う」運用にするか、更新同期を作るか

## 9. 判断が必要なポイント

1. **案A / B / C の選択**（推奨は C）
2. C の場合: フェーズ1の取込ポリシー（全量ミラー or 高評価のみ選抜アーカイブ）
3. ~~Svelte フロント再構築を続けるか、Falcon UI に寄せるか~~（フェーズ2 の分岐）
   → **2026-07-25 追記: 決着済み。Falcon の Plan 224（commit `d7d221f`）で PromptBox のフロント機能 11 ルート
   （一覧 / 詳細 / 編集 / 一括 / ゴミ箱 / Showcase / スマートフォルダ / 統計 / カタログ / 重複 / Gelbooru タグ /
   クイック評価）が Falcon 側へ全移植された。すなわち C-2「UI を Falcon に寄せ、PromptBox を headless 運用」は
   意思決定を待たずに事実上実行済み。**
   これを前提とした backend-rs の機能ロードマップ → `docs/13_backend_roadmap.md`

**残る判断（1・2 に加えて）**: 評価（rating / user_tags / memo）のマスタをどちらに置くか（§8 の最終項目）。
PromptBox がマスタなら変更フィード + 同期状態の記録で一方向同期、Falcon がマスタなら PromptBox 側の
更新系 API は縮小する。**この決定で連携部分の設計が半分変わる**（詳細 → `docs/13_backend_roadmap.md` §6）。
