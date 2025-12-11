# PromptBox iOS App

PromptBox WebアプリケーションのiOS/iPadOSネイティブクライアント。

## 要件

- iOS 17.0+ / iPadOS 17.0+
- Xcode 15+
- Swift 5.9+

## セットアップ

1. Xcodeでプロジェクトを開く
2. Bundle Identifierを設定
3. 開発チームを選択
4. ビルド＆実行

## サーバー接続設定

デフォルトでは `http://localhost:8000/api` に接続します。
設定画面からAPIサーバーのURLを変更できます。

## プロジェクト構成

```text
PromptBox/
├── App/                    # アプリエントリーポイント
├── Core/
│   ├── Network/           # API通信
│   ├── Models/            # データモデル
│   ├── Services/          # ビジネスロジック
│   └── Utilities/         # 設定・ヘルパー
├── Features/
│   ├── Auth/              # 認証
│   ├── Gallery/           # 画像一覧
│   ├── Detail/            # 画像詳細
│   ├── Stats/             # 統計
│   ├── SmartFolders/      # スマートフォルダ
│   ├── Trash/             # ゴミ箱
│   ├── Settings/          # 設定
│   └── Slideshow/         # スライドショー
├── Shared/
│   ├── Components/        # 共通コンポーネント
│   └── Modifiers/         # ViewModifier
└── Resources/             # アセット・設定
```

## 実装済み機能

### 基本機能

- [x] ログイン/ログアウト
- [x] 画像一覧表示（グリッド）
- [x] 画像詳細表示
- [x] 評価・お気に入り
- [x] タグ管理
- [x] メモ
- [x] 検索・フィルター
- [x] 統計表示
- [x] スライドショー

### 拡張機能

- [x] スマートフォルダ
- [x] 一括操作（評価・お気に入り・タグ・削除）
- [x] ゴミ箱（復元・完全削除）
- [x] 画像ナビゲーション（前後移動）

### iPad対応

- [x] Split View（3カラム表示）
- [x] キーボードショートカット
  - `1-5`: 評価設定
  - `0`: 評価クリア
  - `F`: お気に入りトグル
  - `←/→`: 前後の画像へ移動
  - `⌘+Delete`: 削除

### 未実装

- [ ] オフラインキャッシュ
- [ ] 画像ダウンロード（フォトライブラリ保存）

## 技術スタック

- **UI**: SwiftUI
- **状態管理**: @Observable (Swift 5.9 Observation framework)
- **ネットワーク**: URLSession
- **画像キャッシュ**: Kingfisher
- **認証情報保存**: KeychainAccess

## API対応状況

| エンドポイント | 対応状況 |
|---------------|----------|
| POST /api/auth/login | ✅ |
| POST /api/auth/logout | ✅ |
| GET /api/auth/me | ✅ |
| GET /api/images | ✅ |
| GET /api/images/{id} | ✅ |
| PATCH /api/images/{id} | ✅ |
| DELETE /api/images/{id} | ✅ |
| POST /api/images/{id}/restore | ✅ |
| POST /api/bulk/update | ✅ |
| POST /api/bulk/delete | ✅ |
| POST /api/bulk/restore | ✅ |
| GET /api/tags | ✅ |
| GET /api/stats | ✅ |
| GET /api/smart-folders | ✅ |
| GET /api/smart-folders/{id}/images | ✅ |
| DELETE /api/smart-folders/{id} | ✅ |

## 開発

設計ドキュメント: `../docs/09_ios_app_design.md`
