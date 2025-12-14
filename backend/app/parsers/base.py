from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import Enum
from typing import Any


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
    weight_clip: float | None = None
    hash: str | None = None

    def to_dict(self) -> dict[str, Any]:
        result: dict[str, Any] = {"name": self.name, "weight": self.weight}
        if self.weight_clip is not None:
            result["weight_clip"] = self.weight_clip
        if self.hash is not None:
            result["hash"] = self.hash
        return result


@dataclass
class ControlNetInfo:
    model: str
    weight: float = 1.0
    guidance_start: float = 0.0
    guidance_end: float = 1.0
    preprocessor: str | None = None

    def to_dict(self) -> dict[str, Any]:
        result: dict[str, Any] = {
            "model": self.model,
            "weight": self.weight,
            "guidance_start": self.guidance_start,
            "guidance_end": self.guidance_end,
        }
        if self.preprocessor is not None:
            result["preprocessor"] = self.preprocessor
        return result


@dataclass
class EmbeddingInfo:
    name: str
    hash: str | None = None

    def to_dict(self) -> dict[str, Any]:
        result: dict[str, Any] = {"name": self.name}
        if self.hash is not None:
            result["hash"] = self.hash
        return result


@dataclass
class ParsedMetadata:
    # Source info
    source_tool: SourceTool
    model_type: ModelType | None = None
    has_metadata: bool = True

    # Common fields
    positive_prompt: str | None = None
    negative_prompt: str | None = None
    model_name: str | None = None
    sampler_name: str | None = None
    scheduler: str | None = None
    steps: int | None = None
    cfg_scale: float | None = None
    seed: int | None = None

    # Extended data
    loras: list[LoraInfo] = field(default_factory=list)
    controlnets: list[ControlNetInfo] = field(default_factory=list)
    embeddings: list[EmbeddingInfo] = field(default_factory=list)
    model_params: dict[str, Any] = field(default_factory=dict)
    workflow_extras: dict[str, Any] = field(default_factory=dict)

    # Raw data
    raw_metadata: dict[str, Any] | None = None


class MetadataParser(ABC):
    @abstractmethod
    def can_parse(self, png_info: dict[str, Any]) -> bool:
        """Check if this parser can handle the given PNG info."""
        pass

    @abstractmethod
    def parse(self, png_info: dict[str, Any]) -> ParsedMetadata:
        """Parse the metadata and return a ParsedMetadata object."""
        pass


class ParserError(Exception):
    """Parser-specific error."""

    pass
