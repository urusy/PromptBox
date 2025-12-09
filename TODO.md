# TODO

## 未着手

- [ ] ログイン後に元の検索条件付き画面へリダイレクトする機能追加
  - ログイン画面へ遷移する際に`returnUrl`をクエリパラメータに保存
  - ログイン成功後に`returnUrl`へリダイレクト

- [ ] タグ追加時に使用済みタグ候補を表示する機能追加
  - 最近使用したタグを最大10個表示
  - 候補から選択してタグを追加可能に

- [ ] データ量が増えたときのパフォーマンス改善
  - クエリの最適化
  - インデックスの見直し
  - ページネーションの効率化
  - サムネイル遅延読み込み

- [ ] 詳細画面の画像拡大表示でEscキーで閉じるキーバインド追加

## AI提案機能(未精査)

### 画像閲覧・操作系

- [ ] キーボードナビゲーション - 詳細画面で←→キーで前後の画像へ移動
  - **実装方針**: サーバーサイドで前後画像IDを計算し、ページをまたいでナビゲーション可能に
  - **修正ファイル**:
    - `backend/app/schemas/image.py`: ImageResponseに`prev_id`/`next_id`追加
    - `backend/app/services/image_service.py`: `get_image_with_neighbors()`メソッド追加
    - `backend/app/api/endpoints/images.py`: 詳細APIに検索パラメータ受け取り追加
    - `frontend/src/utils/searchParams.ts` (新規): parseSearchParams/toSearchParams共通化
    - `frontend/src/pages/DetailPage.tsx`: キーボードナビゲーション追加
    - `frontend/src/pages/GalleryPage.tsx`: 共通関数をimportに変更
    - `frontend/src/api/images.ts`: get()に検索パラメータ渡し対応
  - **バックエンド処理**:
    - 詳細API呼び出し時に検索条件を受け取る
    - 現在画像のソート値を基準に、前後1件のIDをクエリで取得
    - `prev_id`/`next_id`をレスポンスに含める
  - **フロントエンド処理**:
    - URLの検索パラメータを詳細API呼び出し時に渡す
    - keydownイベントで←→キー検出、テキスト入力中は無視
    - `image.prev_id`/`image.next_id`を使ってnavigate
  - **メリット**: ページ概念不要でシームレスにナビゲーション、スケーラブル
  - **工数目安**: 小〜中
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

- [ ] スライドショー機能 - お気に入りや高評価画像を自動再生
  - **実装方針**: 検索結果またはお気に入りを自動再生するフルスクリーンモード
  - **修正ファイル**:
    - `frontend/src/components/gallery/Slideshow.tsx` (新規): スライドショーコンポーネント
    - `frontend/src/pages/GalleryPage.tsx`: スライドショー開始ボタン追加
  - **処理内容**:
    - フルスクリーン表示で画像を自動切り替え（間隔設定可能: 3-10秒）
    - 一時停止/再生、前後移動のコントロール
    - ESCまたはクリックで終了
  - **工数目安**: 小〜中

### 検索・フィルター系

- [ ] 検索条件の保存・呼び出し - よく使う検索条件をプリセットとして保存
  - **実装方針**: 検索条件をLocalStorageまたはDBに保存、ドロップダウンから呼び出し
  - **修正ファイル**:
    - `frontend/src/components/gallery/SearchPresets.tsx` (新規): プリセット管理UI
    - `frontend/src/stores/searchPresetStore.ts` (新規): Zustandストア
    - `frontend/src/components/gallery/SearchForm.tsx`: プリセット選択UI追加
  - **処理内容**:
    - 現在の検索条件に名前を付けて保存
    - 保存済みプリセットをドロップダウンで選択→条件復元
    - プリセットの編集・削除
  - **工数目安**: 小（フロントエンドのみ、LocalStorage使用時）

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

- [ ] スマートフォルダ - 条件に合う画像を自動収集（例：★4以上のSDXL画像）
  - **実装方針**: 検索条件を保存し、動的に該当画像を表示する仮想フォルダ
  - **修正ファイル**:
    - `backend/app/models/smart_folder.py` (新規): SmartFolderモデル（条件JSON保存）
    - `backend/app/api/endpoints/smart_folders.py` (新規): スマートフォルダAPI
    - `frontend/src/pages/SmartFoldersPage.tsx` (新規): スマートフォルダ一覧
  - **処理内容**:
    - 検索条件（ImageSearchParams相当）を名前付きで保存
    - フォルダ選択時に条件でクエリ実行、該当画像を動的表示
    - 画像追加時に自動で該当フォルダに反映
  - **工数目安**: 中（検索条件保存の仕組みを流用可能）

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

- [ ] 使用統計ダッシュボード - モデル/LoRA/Samplerの使用頻度グラフ
  - **実装方針**: 集計APIを追加し、Chart.js/Rechartsでグラフ表示
  - **修正ファイル**:
    - `backend/app/api/endpoints/stats.py` (新規): 統計集計API
    - `frontend/src/pages/StatsPage.tsx` (新規): ダッシュボード画面
    - `frontend/src/components/stats/` (新規): グラフコンポーネント群
  - **処理内容**:
    - モデル/LoRA/Sampler別の使用回数を集計（GROUP BY）
    - 時系列での生成数推移（日/週/月単位）
    - 円グラフ、棒グラフ、折れ線グラフで可視化
  - **工数目安**: 中（グラフライブラリ導入）

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
