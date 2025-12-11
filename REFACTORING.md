# リファクタリング対象一覧

コードベースのレビュー結果です。

---

## バックエンド（Python/FastAPI）

### ✅ CRITICAL: SQLインジェクション（修正済み）

#### 1. LoRAフィルタの脆弱性
- **ファイル**: `backend/app/services/image_service.py`
- **ステータス**: ✅ 修正済み
- **修正内容**: `json.dumps()` と `cast(JSONB)` を使用してパラメータ化

#### 2. LIKE句のワイルドカード脆弱性
- **ファイル**: `backend/app/services/image_service.py`
- **ステータス**: ✅ 修正済み
- **修正内容**: `escape_like_pattern()` 関数でエスケープ、`ilike(escape="\\")` を使用

#### 3. タグ検索の同様の問題
- **ファイル**: `backend/app/api/endpoints/tags.py`
- **ステータス**: ✅ 修正済み
- **修正内容**: #2と同様の方法で修正

---

### ✅ HIGH: セキュリティ設定（修正済み）

#### 4. CORS設定
- **ファイル**: `backend/app/main.py`
- **ステータス**: ✅ 修正済み（既に環境変数から読み込み）

#### 5. Cookieセキュリティ設定
- **ファイル**: `backend/app/api/endpoints/auth.py`
- **ステータス**: ✅ 修正済み（本番環境でsecure=true）

#### 6. シークレット設定
- **ファイル**: `backend/app/config.py`
- **ステータス**: ✅ 修正済み
- **修正内容**: デフォルトの`secret_key`を空にし、未設定時は`secrets.token_urlsafe()`で自動生成。32文字未満のキーは拒否。

---

### ✅ MEDIUM: パストラバーサル（修正済み）

#### 7. パス検証の強化
- **ファイル**: `backend/app/api/endpoints/duplicates.py`
- **ステータス**: ✅ 修正済み
- **修正内容**: `resolve()`後に`relative_to()`でパス containment をチェック

#### 8. file_utils.pyの改善
- **ファイル**: `backend/app/utils/file_utils.py`
- **ステータス**: ✅ 修正済み
- **修正内容**: シンボリックリンク解決後にcontainmentチェック

---

### ✅ MEDIUM: エラーハンドリング（修正済み）

#### 9. 例外の具体化
- **ファイル**: 複数
- **ステータス**: ✅ 修正済み
- **修正内容**: 具体的な例外型（`OSError`, `IOError`, `RuntimeError`, `KeyError`, `ValueError`等）をキャッチ

#### 10. エラーメッセージの安全化
- **ファイル**: `backend/app/api/endpoints/duplicates.py`
- **ステータス**: ✅ 修正済み
- **修正内容**: 汎用的なエラーメッセージを返すよう変更

---

### ✅ MEDIUM: パフォーマンス（修正済み）

#### 11. N+1問題
- **ファイル**: `backend/app/services/batch_service.py`
- **ステータス**: ✅ 修正済み
- **修正内容**: 永久削除に一括DELETEクエリを使用

#### 12. タグ列挙の最適化
- **ファイル**: `backend/app/api/endpoints/tags.py`
- **ステータス**: ✅ 既に最適化済み（PostgreSQLのJSONB関数使用）

---

### ✅ MEDIUM: スレッドセーフティ（修正済み）

#### 13. Watcherのset操作
- **ファイル**: `backend/app/workers/watcher.py`
- **ステータス**: ✅ 修正済み
- **修正内容**: `ThreadSafeSet`クラスを実装し、`threading.Lock()`で保護

---

### ✅ LOW: コード品質（修正済み）

#### 14. 戻り値の型ヒント不足
- **ファイル**: `backend/app/services/image_service.py`
- **ステータス**: ✅ 修正済み
- **修正内容**: `_build_search_query`に`-> Select[tuple[Image]]`型ヒントを追加

#### 15. setattr によるバリデーションバイパス
- **ファイル**: `backend/app/services/image_service.py`
- **行**: 227
- **ステータス**: 許容
- **理由**: SQLAlchemyモデルの更新であり、Pydanticバリデーションは入力スキーマで既に実行済み

---

## フロントエンド（TypeScript/React）

### ✅ HIGH: セキュリティ（修正済み）

#### 16. オープンリダイレクト脆弱性
- **ファイル**: `frontend/src/pages/LoginPage.tsx`
- **ステータス**: ✅ 修正済み
- **修正内容**: ホワイトリスト方式で許可されたパスのみリダイレクト許可

#### 17. 画像パスのサニタイズ
- **ファイル**: `frontend/src/utils/imagePath.ts` (新規作成)
- **ステータス**: ✅ 修正済み
- **修正内容**: パス検証ユーティリティを作成（`validateImagePath`, `getImageUrl`, `getThumbnailUrl`）

---

### ✅ MEDIUM: パフォーマンス（修正済み）

#### 18. ImageCardのReact.memo
- **ファイル**: `frontend/src/components/gallery/ImageCard.tsx`
- **ステータス**: ✅ 修正済み
- **修正内容**: `React.memo`でラップ、`useCallback`でイベントハンドラをメモ化

#### 19. SearchFormのuseMemo
- **ファイル**: `frontend/src/components/gallery/SearchForm.tsx`
- **ステータス**: ✅ 修正済み
- **修正内容**: `processedModels`を`useMemo`でメモ化

#### 20-21. useCallback追加
- **ファイル**: `frontend/src/pages/DetailPage.tsx`
- **ステータス**: ✅ 既に実装済み（`navigateToImage`, `handleRatingChange`）

#### 22. Paginationのキー修正
- **ファイル**: `frontend/src/components/common/Pagination.tsx`
- **ステータス**: ✅ 修正済み
- **修正内容**: `key={i}`を`key={\`page-${p}\`}`と`key={\`ellipsis-${i}\`}`に変更

---

### 🟡 MEDIUM: 型安全性（許容）

#### 23-24. 型キャスト
- **ファイル**: `frontend/src/pages/DetailPage.tsx`
- **ステータス**: 許容（`e.target as HTMLElement`は安全なキャスト）
- **理由**: DOMイベントのtargetは常にElementであり、HTMLElementへのキャストはTypeScriptの制約

#### 25. Record<string, unknown>
- **ファイル**: `frontend/src/types/image.ts`
- **ステータス**: 許容
- **理由**: `model_params`, `workflow_extras`はバックエンドから動的に返されるJSONで、厳密な型定義は難しい

---

### ✅ LOW: アクセシビリティ（修正済み）

#### 26. Slideshowの空alt
- **ファイル**: `frontend/src/components/gallery/Slideshow.tsx`
- **ステータス**: ✅ 修正済み
- **修正内容**: モデル名または画像番号を表示する意味のあるalt属性を設定

#### 27. ドロップダウンのARIAラベル不足
- **ファイル**:
  - `frontend/src/components/gallery/SearchForm.tsx`
  - `frontend/src/components/detail/TagEditor.tsx`
- **ステータス**: ✅ 修正済み
- **修正内容**: `aria-label`, `aria-expanded`, `aria-haspopup`, `role`属性を追加

#### 28. モーダルのフォーカス管理
- **ファイル**: `frontend/src/components/gallery/SearchForm.tsx`
- **ステータス**: ✅ 修正済み
- **修正内容**: フォーカストラップを実装、`role="dialog"`, `aria-modal`, `aria-labelledby`を追加

---

### ✅ LOW: Reactベストプラクティス（修正済み）

#### 29. useEffectの依存配列
- **ファイル**: `frontend/src/pages/GalleryPage.tsx`
- **ステータス**: ✅ 既に修正済み
- **理由**: `updateUrl`は既に`useCallback`でメモ化されている

#### 30. エラーバウンダリの不在
- **ファイル**: `frontend/src/components/common/ErrorBoundary.tsx` (新規作成)
- **ステータス**: ✅ 修正済み
- **修正内容**: `ErrorBoundary`コンポーネントを作成し、`App.tsx`でアプリ全体をラップ

---

## 優先順位

### ✅ 即時対応（CRITICAL/HIGH）- 完了
1. ✅ SQLインジェクション（#1, #2, #3）
2. ✅ オープンリダイレクト（#16）
3. ✅ CORS設定（#4）
4. ✅ シークレットのハードコード（#6）

### ✅ 次回スプリント（MEDIUM）- 完了
5. ✅ パストラバーサル（#7, #8）
6. ✅ React.memoの追加（#18）
7. ✅ エラーハンドリング改善（#9, #10）
8. ✅ N+1問題（#11）
9. ✅ スレッドセーフティ（#13）

### ✅ バックログ（LOW）- 完了
10. ✅ 型安全性の改善（#23, #24, #25）- 許容として対応済み
11. ✅ アクセシビリティ（#26, #27, #28）
12. ✅ 型ヒントの追加（#14）
13. ✅ エラーバウンダリ（#30）

---

## 参考情報

レビュー実施日: 2024-12-11
レビュー対象: backend/, frontend/, docs/, docker-compose.yml, CLAUDE.md
除外対象: ios/
