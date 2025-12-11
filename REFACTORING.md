# リファクタリング対象一覧

iOSフォルダを除くコードベースのレビュー結果です。

---

## バックエンド（Python/FastAPI）

### 🔴 CRITICAL: SQLインジェクション

#### 1. LoRAフィルタの脆弱性
- **ファイル**: `backend/app/services/image_service.py`
- **行**: 54-57
- **問題**: ユーザー入力がJSON文字列に直接挿入されている
- **現状コード**:
  ```python
  Image.loras.op("@>")(f'[{{"name": "{params.lora_name}"}}]')
  ```
- **攻撃例**: `lora_name = '"}]","test":["malicious'` で任意のJSONを注入可能
- **修正方法**: `cast()` と `literal()` を使用してパラメータ化する

#### 2. LIKE句のワイルドカード脆弱性
- **ファイル**: `backend/app/services/image_service.py`
- **行**: 34
- **問題**: `%` と `_` がエスケープされていない
- **現状コード**:
  ```python
  Image.model_name.ilike(f"%{params.model_name}%")
  ```
- **修正方法**: SQLAlchemyの `escape` 機能を使用、または手動でエスケープ

#### 3. タグ検索の同様の問題
- **ファイル**: `backend/app/api/endpoints/tags.py`
- **行**: 39
- **問題**: 上記と同様

---

### 🟠 HIGH: セキュリティ設定

#### 4. デバッグモードでの弱いCORS設定
- **ファイル**: `backend/app/main.py`
- **行**: 47-55
- **問題**: `allow_origins`, `allow_methods`, `allow_headers` がすべて `["*"]`
- **修正方法**: 環境変数から明示的なオリジンリストを読み込む

#### 5. デバッグモードでの非セキュアCookie
- **ファイル**: `backend/app/api/endpoints/auth.py`
- **行**: 26-34
- **問題**: デバッグ時に `secure=False`, `samesite="lax"`
- **修正方法**: 本番環境以外でも可能な限りセキュアな設定を使用

#### 6. ハードコードされたデフォルトシークレット
- **ファイル**: `backend/app/config.py`
- **行**: 10-12
- **問題**: デフォルトの `secret_key` と `admin_password_hash` がコミットされている
- **修正方法**: デフォルト値を削除し、環境変数必須に

---

### 🟠 MEDIUM: パストラバーサル

#### 7. パス検証の不十分なチェック
- **ファイル**: `backend/app/api/endpoints/duplicates.py`
- **行**: 86
- **問題**: 親ディレクトリのみチェック、シンボリックリンク対策なし
- **修正方法**: `resolve()` 後に `startswith()` でチェック

#### 8. file_utils.pyの同様の問題
- **ファイル**: `backend/app/utils/file_utils.py`
- **行**: 32-39
- **問題**: 文字列比較のみでシンボリックリンク攻撃に脆弱

---

### 🟠 MEDIUM: エラーハンドリング

#### 9. 広すぎる例外キャッチ
- **ファイル**: 複数
  - `backend/app/parsers/factory.py` (行 33-41)
  - `backend/app/workers/watcher.py` (行 76, 97)
  - `backend/app/api/endpoints/duplicates.py` (行 73-74, 96)
  - `backend/app/parsers/png_reader.py` (行 78)
- **問題**: `except Exception as e:` で全例外をキャッチ
- **修正方法**: 特定の例外型をキャッチ

#### 10. 内部エラーメッセージの漏洩
- **ファイル**: `backend/app/api/endpoints/duplicates.py`
- **行**: 97
- **問題**: `detail=str(e)` でシステム情報が漏洩する可能性
- **修正方法**: ジェネリックなエラーメッセージを返す

---

### 🟠 MEDIUM: パフォーマンス

#### 11. バッチ操作でのN+1問題
- **ファイル**: `backend/app/services/batch_service.py`
- **行**: 60-85
- **問題**: 画像ごとに個別の更新クエリを実行
- **修正方法**: 一括更新クエリを使用

#### 12. タグ列挙の非効率性
- **ファイル**: `backend/app/api/endpoints/tags.py`
- **行**: 24-46
- **問題**: 全画像の全タグを展開してからグループ化
- **修正方法**: ページネーションまたはキャッシュの導入

---

### 🟠 MEDIUM: スレッドセーフティ

#### 13. Watcherのset操作
- **ファイル**: `backend/app/workers/watcher.py`
- **行**: 53
- **問題**: `self._processing` セットがロックなしで複数スレッドからアクセス
- **修正方法**: `threading.Lock()` または `queue.Queue` を使用

---

### 🟡 LOW: コード品質

#### 14. 戻り値の型ヒント不足
- **ファイル**: `backend/app/services/image_service.py`
- **行**: 16
- **問題**: `_build_search_query` に戻り値型がない

#### 15. setattr によるバリデーションバイパス
- **ファイル**: `backend/app/services/image_service.py`
- **行**: 227
- **問題**: `setattr(image, key, value)` でPydanticバリデーションをバイパス

---

## フロントエンド（TypeScript/React）

### 🟠 HIGH: セキュリティ

#### 16. オープンリダイレクト脆弱性
- **ファイル**: `frontend/src/pages/LoginPage.tsx`
- **行**: 23
- **問題**: `returnUrl` の検証が不十分（`/` で始まるかのみ）
- **修正方法**: 既知のルートのホワイトリストで検証

#### 17. 画像パスのサニタイズ不足
- **ファイル**: 複数
  - `frontend/src/components/detail/DetailPage.tsx` (行 185, 313-314)
  - その他のImageCard, Slideshow等
- **問題**: APIからのパスを直接使用
- **修正方法**: パス形式のバリデーション追加

---

### 🟠 MEDIUM: パフォーマンス

#### 18. ImageCardにReact.memoがない
- **ファイル**: `frontend/src/components/gallery/ImageCard.tsx`
- **行**: 11
- **問題**: リストレンダリング時に全カードが再レンダリング
- **修正方法**: `React.memo` でラップ

#### 19. SearchFormの再レンダリング
- **ファイル**: `frontend/src/components/gallery/SearchForm.tsx`
- **行**: 98-662
- **問題**: `processedModels` が毎レンダリングで再計算
- **修正方法**: `useMemo` で最適化

#### 20. イベントハンドラのメモ化不足
- **ファイル**: `frontend/src/pages/DetailPage.tsx`
- **行**: 31-33
- **問題**: `handleRatingChange` 等が `useCallback` でラップされていない
- **修正方法**: `useCallback` を使用

#### 21. キーボードイベントリスナーの非効率
- **ファイル**: `frontend/src/pages/DetailPage.tsx`
- **行**: 49-95
- **問題**: 依存配列に頻繁に変わる値が含まれる
- **修正方法**: 依存配列を最適化

#### 22. Paginationのキー問題
- **ファイル**: `frontend/src/components/common/Pagination.tsx`
- **行**: 64-82
- **問題**: `key={i}` でインデックスをキーに使用
- **修正方法**: ユニークな識別子を使用

---

### 🟠 MEDIUM: 型安全性

#### 23. 安全でない型キャスト
- **ファイル**: `frontend/src/components/detail/DetailPage.tsx`
- **行**: 52
- **問題**: `e.target as HTMLElement` の検証なしキャスト
- **修正方法**: `instanceof` でチェック

#### 24. TagEditorの型キャスト
- **ファイル**: `frontend/src/components/detail/TagEditor.tsx`
- **行**: 45, 47
- **問題**: `e.target as Node` の検証なしキャスト

#### 25. Record<string, unknown>の使用
- **ファイル**: `frontend/src/types/image.ts`
- **行**: 44, 45
- **問題**: `model_params`, `workflow_extras` が緩い型
- **修正方法**: 既知のフィールドに対して型を定義

---

### 🟡 LOW: アクセシビリティ

#### 26. Slideshowの空alt
- **ファイル**: `frontend/src/components/gallery/Slideshow.tsx`
- **行**: 144
- **問題**: `alt=""` で画像の説明がない
- **修正方法**: 意味のあるalt属性を設定

#### 27. ドロップダウンのARIAラベル不足
- **ファイル**:
  - `frontend/src/components/gallery/SearchForm.tsx` (行 268-280)
  - `frontend/src/components/detail/TagEditor.tsx` (行 157-183)
- **問題**: `aria-label`, `aria-expanded`, `aria-haspopup` が不足

#### 28. モーダルのフォーカス管理
- **ファイル**: `frontend/src/components/gallery/SearchForm.tsx`
- **行**: 621-653
- **問題**: フォーカストラップがない

---

### 🟡 LOW: Reactベストプラクティス

#### 29. useEffectの依存配列
- **ファイル**: `frontend/src/pages/GalleryPage.tsx`
- **行**: 46-48
- **問題**: `updateUrl` がメモ化されていない

#### 30. エラーバウンダリの不在
- **ファイル**: なし
- **問題**: アプリレベルのエラーバウンダリがない
- **修正方法**: React Error Boundaryコンポーネントを追加

---

## 優先順位

### 即時対応（CRITICAL/HIGH）
1. SQLインジェクション（#1, #2, #3）
2. オープンリダイレクト（#16）
3. CORS設定（#4）
4. シークレットのハードコード（#6）

### 次回スプリント（MEDIUM）
5. パストラバーサル（#7, #8）
6. React.memoの追加（#18）
7. エラーハンドリング改善（#9, #10）
8. N+1問題（#11）
9. スレッドセーフティ（#13）

### バックログ（LOW）
10. 型安全性の改善（#23, #24, #25）
11. アクセシビリティ（#26, #27, #28）
12. 型ヒントの追加（#14）
13. エラーバウンダリ（#30）

---

## 参考情報

レビュー実施日: 2024-12-11
レビュー対象: backend/, frontend/, docs/, docker-compose.yml, CLAUDE.md
除外対象: ios/
