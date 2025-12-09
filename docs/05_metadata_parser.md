# メタデータパーサー設計

## 概要

AI画像生成ツールによって、PNGファイルに埋め込まれるメタデータの形式が異なる。
本システムでは複数のパーサーを用意し、ファクトリパターンで自動判別・パースを行う。

---

## 対応フォーマット

| ツール | 格納場所 | 形式 |
|--------|----------|------|
| ComfyUI | tEXt "prompt", "workflow" | JSON（ノードベース） |
| A1111/Forge | tEXt "parameters" | 独自テキスト形式 |
| NovelAI | tEXt "Comment" | JSON |
| メタデータなし | - | - |

---

## 共通データ構造

### ParsedMetadata

パース結果を格納する共通データクラス。

```python
from dataclasses import dataclass, field
from typing import Optional
from enum import Enum

class SourceTool(str, Enum):
    COMFYUI = "comfyui"
    A1111 = "a1111"
    FORGE = "forge"
    NOVELAI = "novelai"
    UNKNOWN = "unknown"

class ModelType(str, Enum):
    SD15 = "sd15"
    SDXL = "sdxl"
    PONY = "pony"
    ILLUSTRIOUS = "illustrious"
    FLUX = "flux"
    QWEN = "qwen"
    OTHER = "other"

@dataclass
class LoraInfo:
    name: str
    weight: float = 1.0
    weight_clip: Optional[float] = None
    hash: Optional[str] = None

@dataclass
class ControlNetInfo:
    model: str
    weight: float = 1.0
    guidance_start: float = 0.0
    guidance_end: float = 1.0
    preprocessor: Optional[str] = None

@dataclass
class ParsedMetadata:
    # ソース情報
    source_tool: SourceTool
    model_type: Optional[ModelType] = None
    has_metadata: bool = True
    
    # 共通フィールド
    positive_prompt: Optional[str] = None
    negative_prompt: Optional[str] = None
    model_name: Optional[str] = None
    sampler_name: Optional[str] = None
    scheduler: Optional[str] = None
    steps: Optional[int] = None
    cfg_scale: Optional[float] = None
    seed: Optional[int] = None
    
    # 拡張データ
    loras: list[LoraInfo] = field(default_factory=list)
    controlnets: list[ControlNetInfo] = field(default_factory=list)
    embeddings: list[dict] = field(default_factory=list)
    model_params: dict = field(default_factory=dict)
    workflow_extras: dict = field(default_factory=dict)
    
    # 生データ
    raw_metadata: Optional[str] = None
```

---

## パーサー実装

### 基底クラス

```python
from abc import ABC, abstractmethod

class MetadataParser(ABC):
    @abstractmethod
    def can_parse(self, png_info: dict) -> bool:
        """このパーサーで処理可能か判定"""
        pass
    
    @abstractmethod
    def parse(self, png_info: dict) -> ParsedMetadata:
        """メタデータをパース"""
        pass
```

### ComfyUIパーサー

**対象フォーマット:**
```json
{
  "prompt": {
    "3": {
      "class_type": "KSampler",
      "inputs": {
        "seed": 123456,
        "steps": 20,
        "cfg": 7.5,
        "sampler_name": "euler_ancestral",
        "scheduler": "normal",
        ...
      }
    },
    "4": {
      "class_type": "CheckpointLoaderSimple",
      "inputs": {
        "ckpt_name": "ponyDiffusionV6XL.safetensors"
      }
    },
    ...
  },
  "workflow": { ... }
}
```

**判定ロジック:**
- `prompt`キーが存在し、JSONとしてパース可能
- パースしたJSONがノードID→ノード定義のdict構造
- ノード定義に`class_type`キーが存在

**抽出対象ノード:**

| class_type | 抽出フィールド |
|------------|---------------|
| KSampler, KSamplerAdvanced | seed, steps, cfg, sampler_name, scheduler, denoise |
| CheckpointLoaderSimple, CheckpointLoader | ckpt_name → model_name |
| CLIPTextEncode, CLIPTextEncodeSDXL | text → positive/negative_prompt |
| LoraLoader, LoraLoaderModelOnly | lora_name, strength_model, strength_clip |
| ControlNetLoader | control_net_name |
| ControlNetApply, ControlNetApplyAdvanced | strength |

**プロンプト追跡:**
KSamplerのpositive/negative入力は参照（`["6", 0]`形式）なので、参照先ノードを追跡してテキストを取得。

### A1111/Forgeパーサー

**対象フォーマット:**

```text
1girl, beautiful, masterpiece
Negative prompt: ugly, blurry
Steps: 20, Sampler: DPM++ 2M Karras, CFG scale: 7, Seed: 123456, Size: 1024x1024, Model hash: abc123, Model: ponyDiffusionV6XL, VAE: sdxl_vae.safetensors, Clip skip: 2, Lora hashes: "animeMix: abc123"
```

**判定ロジック:**
- `parameters`キーが存在
- テキスト内に`Steps:`と`Sampler:`が含まれる

**抽出ロジック:**
1. 最初の行 → positive_prompt
2. `Negative prompt:`で始まる行 → negative_prompt
3. `Steps:`を含む行をパラメータ行として解析
4. `<lora:name:weight>`パターンでLoRA抽出

**パラメータ行の解析:**

```text
key: value, key: value, ...
```

形式をパース。主要キー:
- Steps → steps
- Sampler → sampler_name
- CFG scale → cfg_scale
- Seed → seed
- Model → model_name
- Model hash → model_hash
- Clip skip → clip_skip (model_params)
- VAE → vae (model_params)

### NovelAIパーサー

**対象フォーマット:**
```json
{
  "prompt": "1girl, beautiful",
  "uc": "ugly, blurry",
  "steps": 28,
  "scale": 5.0,
  "seed": 123456,
  "sampler": "k_euler_ancestral",
  "width": 1024,
  "height": 1024
}
```

**判定ロジック:**
- `Comment`キーが存在
- JSONとしてパース可能
- `uc`キーまたはNovelAI固有のキーが存在

**マッピング:**
- prompt → positive_prompt
- uc → negative_prompt
- scale → cfg_scale
- sampler → sampler_name（`k_`プレフィックス除去）

---

## パーサーファクトリ

```python
class MetadataParserFactory:
    def __init__(self):
        # 優先度順にパーサーを登録
        self.parsers: list[MetadataParser] = [
            ComfyUIParser(),
            A1111Parser(),
            NovelAIParser(),
        ]
    
    def parse(self, png_info: dict) -> ParsedMetadata:
        """適切なパーサーを選択してパース"""
        for parser in self.parsers:
            if parser.can_parse(png_info):
                return parser.parse(png_info)
        
        # どのパーサーにもマッチしない
        return ParsedMetadata(
            source_tool=SourceTool.UNKNOWN,
            has_metadata=False
        )
```

---

## モデルタイプ検出

モデル名からモデルタイプを推定するロジック。

```python
def detect_model_type(model_name: Optional[str]) -> ModelType:
    if not model_name:
        return ModelType.OTHER
    
    name_lower = model_name.lower()
    
    # 検出ルール（優先度順）
    rules = [
        (["qwen"], ModelType.QWEN),
        (["flux"], ModelType.FLUX),
        (["pony"], ModelType.PONY),
        (["illustrious", "noob"], ModelType.ILLUSTRIOUS),
        (["xl", "sdxl"], ModelType.SDXL),
        (["sd15", "v1-5", "1.5", "sd_1"], ModelType.SD15),
    ]
    
    for keywords, model_type in rules:
        if any(kw in name_lower for kw in keywords):
            return model_type
    
    return ModelType.OTHER
```

---

## PNG情報の読み取り

Pillowを使用してPNGのtEXtチャンクを読み取る。

```python
from PIL import Image
from PIL.PngImagePlugin import PngInfo

def read_png_info(file_path: str) -> dict:
    """PNGファイルからメタデータを読み取る"""
    with Image.open(file_path) as img:
        png_info = {}
        
        if hasattr(img, 'info'):
            for key, value in img.info.items():
                png_info[key] = value
        
        return png_info
```

**返却される主なキー:**
- `prompt` - ComfyUIのプロンプトJSON
- `workflow` - ComfyUIのワークフローJSON
- `parameters` - A1111形式のパラメータ文字列
- `Comment` - NovelAI形式のJSON

---

## エラーハンドリング

パース中のエラーは可能な限り吸収し、取得できた情報のみを返す。

```python
class ParserError(Exception):
    """パーサー固有のエラー"""
    pass

def safe_parse(parser: MetadataParser, png_info: dict) -> ParsedMetadata:
    """エラーを吸収してパース"""
    try:
        return parser.parse(png_info)
    except Exception as e:
        logger.warning(f"Parse error: {e}")
        return ParsedMetadata(
            source_tool=SourceTool.UNKNOWN,
            has_metadata=False,
            workflow_extras={"parse_error": str(e)}
        )
```

---

## 拡張方針

新しいツール/フォーマットに対応する場合:

1. `MetadataParser`を継承した新しいパーサークラスを作成
2. `can_parse()`で判定ロジックを実装
3. `parse()`で抽出ロジックを実装
4. `MetadataParserFactory`のパーサーリストに追加

モデル名パターンの追加:

1. `detect_model_type()`のルールリストに追加
