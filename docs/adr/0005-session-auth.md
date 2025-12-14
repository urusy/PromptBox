# ADR-0005: Cookieベースセッション認証の採用

## ステータス

採用

## コンテキスト

Prompt Boxは単一ユーザー向けのローカルアプリケーションとして設計されている。認証方式を決定する必要がある。

要件：

1. シンプルな認証フロー
2. セキュアなセッション管理
3. 開発環境でのテスト容易性
4. モバイルブラウザ対応

## 決定

**JWTトークンをHttpOnly Cookieに保存するセッション認証**を採用する。

### 実装詳細

```python
# ログイン時
response.set_cookie(
    key="session",
    value=jwt_token,
    httponly=True,        # JavaScriptからアクセス不可
    secure=not debug,     # 本番はHTTPSのみ
    samesite="strict",    # CSRFプロテクション
    max_age=60*60*24*7,   # 1週間有効
)
```

### JWTペイロード

```json
{
  "sub": "admin",
  "exp": 1234567890,
  "iat": 1234560000
}
```

## 結果

### 良い影響

- **XSS保護**: HttpOnlyによりJavaScriptからトークンにアクセス不可
- **CSRF保護**: SameSite属性による自動保護
- **シンプル**: クライアント側でのトークン管理不要
- **ステートレス**: サーバー側でセッションストレージ不要

### 悪い影響

- ログアウト時の即座のトークン無効化が困難（JWTの特性）
- モバイルネイティブアプリでの利用時に追加考慮が必要

## セキュリティ考慮事項

### 設定

| 属性 | 本番環境 | 開発環境 |
|------|---------|---------|
| httponly | true | true |
| secure | true | false |
| samesite | strict | lax |

### パスワード保存

- bcryptによるハッシュ化
- cost factor: 12
- 環境変数で事前設定したハッシュを使用

```bash
# パスワードハッシュ生成
python -c "import bcrypt; print(bcrypt.hashpw(b'password', bcrypt.gensalt()).decode())"
```

## 代替案

### LocalStorage + Bearer Token

```javascript
localStorage.setItem('token', jwt);
// リクエスト時
headers: { 'Authorization': `Bearer ${token}` }
```

- クライアント側での実装が必要
- XSS脆弱性時にトークン漏洩リスク

### セッションID + サーバーストレージ

- ステートフルなセッション管理
- スケーラビリティの問題（複数サーバー時）

### OAuth 2.0

- 外部認証プロバイダー依存
- 単一ユーザーアプリには過剰

## 参考

- [OWASP Session Management](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- [JWT Best Practices](https://auth0.com/blog/jwt-json-webtoken-best-practices/)
