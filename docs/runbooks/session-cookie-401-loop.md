# ランブック: ログインしても全 API が 401 で `/login` にループする

## 症状のシグネチャ

- `http://<NAS-IP>:<PORT>`（LAN 直アクセス、平文 HTTP）で開くと、
  `/api/images` `/api/tags` `/api/stats/*` など**すべての API が 401 `{"detail":"Not authenticated"}`**
- フロントは `frontend/src/api/client.ts` の interceptor により `/login` へ遷移する
- ログインしても Cookie が残らず、同じループに戻る
- **HTTPS 経由（Cloudflare Tunnel のドメイン）では正常に動く**——この非対称性が決め手

## 決め手になった観測（2026-07-24）

| 確認 | 結果 |
|---|---|
| `curl -i http://<NAS-IP>:<PORT>/api/health` | 200 → バックエンドは生存 |
| `curl -i .../api/auth/login`（誤資格情報） | 401 `Invalid username or password` → 認証機構は正常 |
| 配信中の bundle 名 vs `frontend/dist` | 一致 → フロントは最新（デプロイ漏れではない） |
| `grep '^DEBUG=' /volume1/docker/prompt_box/.env` | **出力なし** → `DEBUG` 未設定 |
| nginx の X-Forwarded-Proto 伝播（修正前） | `-H 'X-Forwarded-Proto: https'` を送っても上流には `http` として届く |

## 根本原因

`DEBUG` 未設定 → `config.rs` の `get_bool("DEBUG", false)` により `debug=false` →
セッション Cookie に `Secure` が付く（`http/auth.rs`）。
**ブラウザは localhost 以外の平文 HTTP オリジンでは `Secure` Cookie を保存しない**ため、
ログインに成功しても Cookie が残らず、以降の全リクエストが 401 になる。

加えて `frontend/nginx.conf` が `proxy_set_header X-Forwarded-Proto $scheme;` で
上流のヘッダを無条件に上書きしていたため、Cloudflare Tunnel 経由の HTTPS も
バックエンドには `http` として見えていた。

## 修正

リクエストごとに `Secure` を決める方式にした（HTTPS と LAN 平文 HTTP の両立）。

- `backend-rs/src/http/auth.rs`
  - `request_is_https()`: `X-Forwarded-Proto`（先頭ホップ）→ `CF-Visitor` の順で判定
  - `cookie_secure()`: `SESSION_COOKIE_SECURE` の明示指定 > 自動判定（`!debug && https`）
  - `logout` も同じ属性で削除 Cookie を発行（属性不一致だと Cookie が消えない）
- `frontend/nginx.conf`: `map $http_x_forwarded_proto` で上流の値を尊重し、無ければ `$scheme`
- `.env.example` / `docker-compose*.yml`: `SESSION_COOKIE_SECURE`（未設定＝自動）を追加

回帰テスト: `http::auth::tests::login_over_plain_http_omits_secure` ほか計 9 件。
修正を戻すと 2 件が Red になることを確認済み。

## 次に最速で同じ結論へ至る確認手順

```bash
# 1) バックエンドは生きているか
curl -si http://<HOST>/api/health | head -1

# 2) ログイン応答の Cookie 属性を見る（ここに Secure があり、かつ http で開いていれば確定）
read -rs -p "password: " P; echo
curl -si -X POST http://<HOST>/api/auth/login -H 'Content-Type: application/json' \
  --data-binary "$(printf '{"username":"admin","password":"%s"}' "$P")" \
  | grep -iE '^(HTTP/|set-cookie)'; unset P

# 3) nginx が上流の scheme を潰していないか
curl -s -H 'X-Forwarded-Proto: https' http://<HOST>/api/health -o /dev/null -w '%{http_code}\n'
```

## 応急処置（再ビルドせずに回避する場合）

NAS の `.env` に一行足して `up -d --force-recreate`:

```env
SESSION_COOKIE_SECURE=false
```

HTTPS 経由でも `Secure` が外れるため、恒久策は上記の自動判定版をデプロイすること。
