# ADR-0003: メタデータパーサーのストラテジーパターン

## ステータス

採用

## コンテキスト

画像生成AIツール（ComfyUI、A1111/Forge、NovelAI）はそれぞれ異なる形式でメタデータをPNGファイルに埋め込む：

- **ComfyUI**: JSON形式のワークフローとプロンプトデータ
- **A1111/Forge**: 独自のテキスト形式（"Steps: 20, Sampler: ..."）
- **NovelAI**: JSONコメント形式

これらを効率的にパースし、統一されたデータ構造に変換する必要がある。

## 決定

**ストラテジーパターン**を採用し、各ツール用のパーサーをプラグイン可能な形式で実装する。

### 構造

```
MetadataParserFactory
├── ComfyUIParser
├── A1111Parser
└── NovelAIParser
```

### インターフェース

```python
class MetadataParser(ABC):
    @abstractmethod
    def can_parse(self, png_info: dict[str, Any]) -> bool:
        """このパーサーで処理可能か判定"""
        pass

    @abstractmethod
    def parse(self, png_info: dict[str, Any]) -> ParsedMetadata:
        """メタデータをパースして統一形式に変換"""
        pass
```

### ファクトリー

```python
class MetadataParserFactory:
    def __init__(self) -> None:
        self.parsers = [
            ComfyUIParser(),
            A1111Parser(),
            NovelAIParser(),
        ]

    def parse(self, png_info: dict[str, Any]) -> ParsedMetadata:
        for parser in self.parsers:
            if parser.can_parse(png_info):
                return parser.parse(png_info)
        return ParsedMetadata(source_tool=SourceTool.UNKNOWN)
```

## 結果

### 良い影響

- **拡張性**: 新しいツール対応時に新パーサークラスを追加するだけ
- **テスト容易性**: 各パーサーを独立してテスト可能
- **単一責任原則**: 各パーサーは1つのフォーマットのみを担当
- **優先順位制御**: パーサーの順序で優先度を設定可能

### 悪い影響

- クラス数が増加
- 共通処理の重複が発生する可能性

## 代替案

### 単一クラスでの条件分岐

```python
def parse(png_info):
    if "prompt" in png_info:
        return parse_comfyui(png_info)
    elif "parameters" in png_info:
        return parse_a1111(png_info)
    ...
```

- シンプルだが拡張性が低い
- テストが困難

### 正規表現ベースの統一パーサー

- 複雑な正規表現が必要
- メンテナンスが困難

## 参考

- [Strategy Pattern - Refactoring Guru](https://refactoring.guru/design-patterns/strategy)
- [Factory Method Pattern](https://refactoring.guru/design-patterns/factory-method)
