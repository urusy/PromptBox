# ADR-0004: 画像ストレージの分散配置構造

## ステータス

採用

## コンテキスト

大量の画像ファイル（数万〜数十万枚）を効率的に保存・アクセスする必要がある。

考慮事項：

1. ファイルシステムの制限（1ディレクトリ内のファイル数上限）
2. 元画像とサムネイルの管理
3. ファイルパスからの情報推測防止
4. バックアップの容易さ

## 決定

**UUID v7ベースの分散ディレクトリ構造**を採用する。

### ディレクトリ構造

```
storage/
├── original/           # 元画像
│   ├── 01/
│   │   ├── 8d/
│   │   │   └── 018d9e0a-1234-7abc-8def-1234567890ab.png
│   │   └── 8e/
│   └── 02/
└── thumbnail/          # サムネイル（WebP形式）
    ├── 01/
    │   ├── 8d/
    │   │   └── 018d9e0a-1234-7abc-8def-1234567890ab.webp
    │   └── 8e/
    └── 02/
```

### パス生成ロジック

```python
def get_storage_path(image_id: UUID, storage_type: str = "original") -> Path:
    id_hex = image_id.hex
    # 最初の2文字と次の2文字でサブディレクトリを作成
    subdir1 = id_hex[:2]
    subdir2 = id_hex[2:4]
    ext = ".png" if storage_type == "original" else ".webp"
    return Path(f"storage/{storage_type}/{subdir1}/{subdir2}/{image_id}{ext}")
```

## 結果

### 良い影響

- **スケーラビリティ**: 1ディレクトリあたり最大256ファイル（16×16サブディレクトリ）
- **均等分散**: UUID v7の時間ベース性質により、自然に分散される
- **推測防止**: UUIDベースのためパスからの情報漏洩を防止
- **バックアップ容易**: ディレクトリ単位での増分バックアップが可能

### 悪い影響

- ファイルパスの解決に追加の計算が必要
- ディレクトリ階層が深くなる

## 代替案

### フラットディレクトリ

```
storage/original/018d9e0a-1234-7abc-8def-1234567890ab.png
```

- シンプル
- しかしファイル数増加時にパフォーマンス低下

### 日付ベースディレクトリ

```
storage/2024/01/15/image_001.png
```

- 日付での検索に便利
- しかし1日に大量の画像がある場合に問題

### ハッシュベースディレクトリ

```
storage/ab/cd/abcdef1234567890.png
```

- 均等分散
- しかしハッシュ計算のオーバーヘッド

## 参考

- [Git Object Storage](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)
- [S3 Key Naming Best Practices](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-keys.html)
