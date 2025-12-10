# TODO

## 未着手

- [ ] 評価傾向分析 - どの設定が高評価になりやすいか分析
  - **実装方針**: 評価と各設定値のクロス集計、相関分析を表示
  - **修正ファイル**:
    - `backend/app/api/endpoints/stats.py`: 評価相関APIエンドポイント追加
    - `frontend/src/pages/StatsPage.tsx`: 評価分析セクション追加
  - **処理内容**:
    - 設定値（モデル/LoRA/CFG/Steps等）×平均評価のヒートマップ
    - 高評価画像で頻出する設定値のランキング
    - 「おすすめ設定」の自動提案（統計ベース）
  - **工数目安**: 中（統計ダッシュボードの拡張）
  - **依存**: 使用統計ダッシュボード

- [ ] データ量が増えたときのパフォーマンス改善
  - クエリの最適化
  - インデックスの見直し
  - ページネーションの効率化
  - サムネイル遅延読み込み

- [ ] リポジトリをpuclicへ変更してCode Rabbitを導入する

## AI提案機能(未精査)

### 画像閲覧・操作系

- [ ] 画像比較モード - 2枚の画像を並べて比較表示
  - **実装方針**: 選択モードで2枚選択→比較ボタンで専用モーダル/ページ表示
  - **修正ファイル**:
    - `frontend/src/components/gallery/CompareModal.tsx` (新規): 比較表示コンポーネント
    - `frontend/src/components/gallery/SelectionToolbar.tsx`: 比較ボタン追加
  - **処理内容**:
    - 2枚の画像を左右に並べて表示（同期スクロール対応）
    - メタデータの差分をハイライト表示
    - スワイプ/オーバーレイ切り替えで差異確認
  - **工数目安**: 中（UIコンポーネント実装がメイン）

### 検索・フィルター系

- [ ] 類似画像検索 - 選択した画像に似た画像を表示
  - **実装方針**: 画像の特徴ベクトル（CLIP等）をDB保存し、コサイン類似度で検索
  - **修正ファイル**:
    - `backend/app/services/similarity_service.py` (新規): 類似度計算
    - `backend/app/api/endpoints/images.py`: 類似画像APIエンドポイント追加
    - `backend/app/models/image.py`: 特徴ベクトルカラム追加
    - `frontend/src/pages/DetailPage.tsx`: 「類似画像を探す」ボタン追加
  - **処理内容**:
    - 画像取り込み時にCLIP等で特徴ベクトル抽出・保存
    - 選択画像の特徴ベクトルと全画像を比較、上位N件を返却
    - pgvector拡張でベクトル検索を高速化
  - **工数目安**: 大（ML依存、pgvector導入が必要）

- [ ] LoRA/モデル別のグルーピング表示 - 使用モデルやLoRAごとにグループ化
  - **実装方針**: 新しい一覧表示モードとして、モデル/LoRAでグループ化した表示を追加
  - **修正ファイル**:
    - `backend/app/api/endpoints/images.py`: グループ集計APIエンドポイント追加
    - `frontend/src/pages/GalleryPage.tsx`: 表示モード切り替え追加
    - `frontend/src/components/gallery/GroupedImageGrid.tsx` (新規): グループ表示
  - **処理内容**:
    - モデル名/LoRA名でGROUP BY集計→各グループの画像数とサムネイル取得
    - グループをカード形式で表示、クリックでそのグループの画像一覧へ
    - グループ内でさらに絞り込み可能
  - **工数目安**: 中

### 整理・管理系

- [ ] コレクション/アルバム機能 - 画像をカスタムフォルダに整理
  - **実装方針**: コレクションテーブルを追加し、画像との多対多関係を構築
  - **修正ファイル**:
    - `backend/app/models/collection.py` (新規): Collectionモデル
    - `backend/app/models/image_collection.py` (新規): 中間テーブル
    - `backend/app/api/endpoints/collections.py` (新規): コレクションCRUD API
    - `frontend/src/pages/CollectionsPage.tsx` (新規): コレクション一覧
    - `frontend/src/components/gallery/AddToCollectionModal.tsx` (新規)
  - **処理内容**:
    - コレクション作成（名前、説明、カバー画像）
    - 画像を1つ以上のコレクションに追加/削除
    - コレクション内の画像一覧表示
  - **工数目安**: 中（DBスキーマ変更あり）

- [ ] 画像の並び替え - ドラッグ&ドロップで手動並び替え
  - **実装方針**: コレクション内の画像に表示順序(sort_order)を持たせる
  - **修正ファイル**:
    - `backend/app/models/image_collection.py`: sort_orderカラム追加
    - `backend/app/api/endpoints/collections.py`: 並び順更新API追加
    - `frontend/src/components/gallery/SortableImageGrid.tsx` (新規): D&D対応グリッド
  - **処理内容**:
    - react-beautiful-dnd等でドラッグ&ドロップ実装
    - ドロップ時にsort_orderを一括更新
    - コレクション外（通常一覧）では無効
  - **工数目安**: 中（D&Dライブラリ導入）

### プロンプト活用系

- [ ] プロンプトテンプレート保存 - 良いプロンプトを再利用可能に保存
  - **実装方針**: プロンプトをテンプレートとしてDB保存、カテゴリ分類可能に
  - **修正ファイル**:
    - `backend/app/models/prompt_template.py` (新規): テンプレートモデル
    - `backend/app/api/endpoints/prompt_templates.py` (新規): テンプレートCRUD API
    - `frontend/src/pages/PromptTemplatesPage.tsx` (新規): テンプレート管理画面
    - `frontend/src/pages/DetailPage.tsx`: 「テンプレートとして保存」ボタン追加
  - **処理内容**:
    - 画像詳細からプロンプト（positive/negative）をテンプレート保存
    - カテゴリ/タグで整理、検索可能
    - テンプレートをクリップボードにコピー
  - **工数目安**: 小〜中

- [ ] プロンプト差分表示 - 2枚の画像のプロンプト差分をハイライト
  - **実装方針**: 画像比較モードの一部として、テキストdiffを表示
  - **修正ファイル**:
    - `frontend/src/components/gallery/PromptDiff.tsx` (新規): diff表示コンポーネント
    - `frontend/src/components/gallery/CompareModal.tsx`: PromptDiff組み込み
  - **処理内容**:
    - diff-match-patch等でプロンプトの差分計算
    - 追加部分を緑、削除部分を赤でハイライト
    - positive/negativeそれぞれで差分表示
  - **工数目安**: 小（画像比較モードの拡張）
  - **依存**: 画像比較モード

- [ ] ワイルドカード展開履歴 - 使用されたワイルドカードの記録
  - **実装方針**: メタデータパース時にワイルドカード情報を抽出・保存（対応ツール限定）
  - **修正ファイル**:
    - `backend/app/parsers/`: 各パーサーでワイルドカード情報抽出
    - `backend/app/models/image.py`: wildcard_historyカラム追加（JSONB）
    - `frontend/src/pages/DetailPage.tsx`: ワイルドカード履歴表示
  - **処理内容**:
    - A1111/ForgeのDynamic Prompts拡張のメタデータから展開前後を抽出
    - 詳細画面で展開前テンプレート→展開後プロンプトを対比表示
  - **工数目安**: 中（メタデータフォーマット調査が必要）
  - **制限**: Dynamic Prompts拡張使用時のみ対応

### 統計・分析系

### その他

- [ ] ダークモード/ライトモード切り替え
  - **実装方針**: TailwindCSSのdarkモードを活用、テーマ設定をLocalStorage保存
  - **修正ファイル**:
    - `frontend/tailwind.config.js`: darkMode設定
    - `frontend/src/stores/themeStore.ts` (新規): テーマ状態管理
    - `frontend/src/components/common/ThemeToggle.tsx` (新規): 切り替えボタン
    - `frontend/src/App.tsx`: テーマクラス適用
    - 各コンポーネント: `dark:` プレフィックスでダークスタイル追加
  - **処理内容**:
    - システム設定に従うauto / light / darkの3モード
    - ヘッダーにトグルボタン配置
    - 選択をLocalStorage保存、次回起動時に復元
  - **工数目安**: 中（全コンポーネントのスタイル調整が必要）
  - **備考**: 現在はダークモード固定、ライトモード追加が主な作業

- [ ] 画像のクラウドバックアップ連携（Google Drive等）
  - **実装方針**: OAuth認証でクラウドストレージAPI連携、選択画像をアップロード
  - **修正ファイル**:
    - `backend/app/services/cloud_backup_service.py` (新規): クラウドAPI連携
    - `backend/app/api/endpoints/backup.py` (新規): バックアップAPI
    - `frontend/src/pages/SettingsPage.tsx`: クラウド連携設定UI
    - `frontend/src/components/gallery/SelectionToolbar.tsx`: バックアップボタン追加
  - **処理内容**:
    - Google Drive / Dropbox等のOAuth認証フロー
    - 選択画像を指定フォルダにアップロード
    - バックアップ履歴の管理
  - **工数目安**: 大（OAuth実装、各クラウドAPI対応）
  - **セキュリティ**: アクセストークンの安全な保存が必要

## 完了済み

### 最近の更新

- [x] 使用統計ダッシュボード機能を追加
  - /stats ページで統計情報を表示
  - 概要: 総画像数、お気に入り数、評価済み数、平均評価
  - Model Type / Source Tool の分布（円グラフ）
  - Rating分布（棒グラフ）
  - Top Models / Samplers / LoRAs（横棒グラフ）
  - 日次生成数推移（折れ線グラフ、30日間）
  - Rechartsライブラリを導入
- [x] ログアウトボタンをユーザーメニューに移動
  - ユーザー名クリックでドロップダウンメニュー表示
  - メニュー内にLogoutボタン配置
- [x] 一覧を開いたときにプリセットが選択状態になる問題を修正
  - フィルター条件がない初期状態ではプリセット選択を行わないよう修正
- [x] スマートフォルダ機能を追加
  - 検索条件を保存し、動的に該当画像を表示する仮想フォルダ
  - フォルダ一覧ページ（/smart-folders）を追加
  - フォルダクリックでギャラリーページにフィルター適用して遷移
  - フォルダの作成・編集・削除機能
  - ナビゲーションにSmartリンク追加
- [x] プリセット選択時に適用中のプリセットがわかる表示を追加
  - プリセットボタンに選択中のプリセット名を表示
  - ボタンの背景色を青色に変更（選択中）
  - ドロップダウン内で選択中のプリセットをハイライト
  - 検索条件が変更されプリセットと一致しなくなると自動でクリア
- [x] タグ追加時に使用済みタグ候補を表示する機能追加
  - 最近使用したタグを最大10個表示
  - 候補から選択してタグを追加可能に
  - キーボード操作（↑↓キー、Enter）対応
- [x] プリセット選択時のnull値によるエラー修正
  - APIレスポンスの`null`値が`toSearchParams`でエラーを起こす問題を修正
- [x] 更新日時（Date Updated）でのソート機能追加
- [x] 検索条件の保存・呼び出し機能 - よく使う検索条件をプリセットとして保存
  - DB永続化によるプリセット管理
  - 検索フォームにドロップダウンUI追加
  - プリセットの作成・削除機能
  - プリセット選択時のURL同期
- [x] 詳細画面でfキーでお気に入り、cキーで改善対象のトグル
- [x] 選択ツールバーの改善 - Selectボタン押下で即表示、未選択時は操作ボタン非活性
- [x] 検索フォームUIの改善 - フィルター開閉ボタンと条件削除ボタンの配置変更
- [x] 詳細画面で数字キー(0-5)によるレーティング設定機能追加
- [x] 詳細画面の前後ナビゲーション修正 - 検索条件を維持して遷移・戻り
- [x] レーティングフィルターUIの改善 - 「Any」と「★0 (Unrated)」を分離
- [x] スライドショー機能 - 検索結果を自動再生するフルスクリーンモード
- [x] ログイン後に元の検索条件付き画面へリダイレクトする機能追加
- [x] キーボードナビゲーション - 詳細画面で←→キーで前後の画像へ移動
- [x] 詳細画面の画像拡大表示でEscキーで閉じるキーバインド追加
- [x] 画面下部の選択メニューのモバイル対応（見切れ修正）
- [x] レーティングの日本語ツールチップ追加
- [x] 検索条件追加: Upscale、最小Width/Height、レーティング同等/以上
- [x] 検索条件をURLクエリパラメータに保存
- [x] デプロイメントガイド作成 (docs/07_deployment.md)
- [x] NAS本番用docker-compose.prod.yml追加

### 過去の実装

- [x] UI改善とメタデータパース修正
- [x] ローカルネットワークアクセス対応（モバイル向け）
- [x] アプリ名変更: ComfyUI Gallery → Prompt Box
- [x] XYZ Grid画像対応 (A1111/Forge)
- [x] 重複画像管理ページ追加
- [x] Hires Upscaler検出・表示
- [x] 画像インポート・ストレージ問題修正
- [x] Dockerビルド問題修正
- [x] Phase 4: 拡張機能（評価・タグ・一括操作・エクスポート・ゴミ箱）
- [x] Phase 3: UI強化（一覧・詳細・検索）
- [x] Phase 2: メタデータパーサー・画像ウォッチャー
- [x] Phase 1: 基盤構築（Docker・FastAPI・DB・認証）
