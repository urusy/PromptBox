# ADR-0002: UUID v7をプライマリキーとして使用

## ステータス

採用

## コンテキスト

画像レコードのプライマリキーとして使用するID形式を決定する必要がある。選択肢：

1. **シーケンシャルID（SERIAL/BIGSERIAL）**: PostgreSQLの自動増分
2. **UUID v4**: ランダム生成
3. **UUID v7**: タイムスタンプベース + ランダム
4. **ULID**: Universally Unique Lexicographically Sortable Identifier
5. **Snowflake ID**: Twitter方式の分散ID

## 決定

**UUID v7** をプライマリキーとして採用する。

### 理由

1. **時系列ソート可能**: UUIDにタイムスタンプが埋め込まれているため、作成順でのソートが効率的
2. **B-treeインデックス効率**: 連続的な値によりインデックスの断片化を抑制
3. **分散生成安全**: 中央サーバーなしで衝突のないID生成が可能
4. **標準形式**: RFC準拠のUUID形式で互換性が高い
5. **Python実装**: `uuid-utils`ライブラリで簡単に生成可能

## 結果

### 良い影響

- 画像の作成順でのソートが自然に行える
- インデックスのパフォーマンスがUUID v4より良好
- 将来的な分散システムへの拡張が容易
- URLやAPIで扱いやすい標準形式

### 悪い影響

- UUID v4と比較してタイムスタンプ情報が漏洩する（セキュリティ上の考慮が必要な場合）
- 128ビットのためBIGINTより大きい

## 代替案

### SERIAL/BIGSERIAL
- シンプルで効率的
- しかし分散環境で衝突の可能性
- IDから情報が推測可能（セキュリティ懸念）

### UUID v4
- 完全にランダムで予測不可能
- しかしB-treeインデックスの断片化が発生
- ソート順序が意味を持たない

### ULID
- UUID v7と類似の特性
- しかし標準化されていない
- Base32エンコードでURLセーフ

## 実装例

```python
from uuid_utils import uuid7

# 新しいUUID v7を生成
image_id = uuid7()
# => UUID('018d9e0a-1234-7abc-8def-1234567890ab')
```

## 参考

- [RFC 9562 - UUIDs](https://www.rfc-editor.org/rfc/rfc9562)
- [uuid-utils Python Package](https://github.com/oittaa/uuid-utils)
