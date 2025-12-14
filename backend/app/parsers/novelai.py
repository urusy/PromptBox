import contextlib
import json
from typing import Any

from app.parsers.base import (
    MetadataParser,
    ModelType,
    ParsedMetadata,
    SourceTool,
)


class NovelAIParser(MetadataParser):
    """Parser for NovelAI metadata format."""

    def can_parse(self, png_info: dict[str, Any]) -> bool:
        """Check if the PNG info contains NovelAI metadata."""
        if "Comment" not in png_info:
            return False

        try:
            comment = png_info["Comment"]
            data = json.loads(comment) if isinstance(comment, str) else comment

            # Check for NovelAI-specific keys
            return isinstance(data, dict) and ("uc" in data or "prompt" in data)
        except (json.JSONDecodeError, TypeError):
            return False

    def parse(self, png_info: dict[str, Any]) -> ParsedMetadata:
        """Parse NovelAI metadata."""
        comment_str = png_info.get("Comment", "{}")
        data = json.loads(comment_str) if isinstance(comment_str, str) else comment_str

        metadata = ParsedMetadata(
            source_tool=SourceTool.NOVELAI,
            has_metadata=True,
            raw_metadata=data,
            model_type=ModelType.OTHER,  # NovelAI uses custom models
        )

        # Extract prompts
        if "prompt" in data:
            metadata.positive_prompt = str(data["prompt"])

        if "uc" in data:
            metadata.negative_prompt = str(data["uc"])

        # Extract parameters
        if "steps" in data:
            with contextlib.suppress(ValueError, TypeError):
                metadata.steps = int(data["steps"])

        if "scale" in data:
            with contextlib.suppress(ValueError, TypeError):
                metadata.cfg_scale = float(data["scale"])

        if "seed" in data:
            with contextlib.suppress(ValueError, TypeError):
                metadata.seed = int(data["seed"])

        if "sampler" in data:
            # NovelAI uses k_ prefix for samplers
            sampler = str(data["sampler"])
            if sampler.startswith("k_"):
                sampler = sampler[2:]
            metadata.sampler_name = sampler

        # Store additional NovelAI-specific data in model_params
        model_params: dict[str, Any] = {}

        for key in ["width", "height", "n_samples", "ucPreset", "qualityToggle"]:
            if key in data:
                model_params[key] = data[key]

        if model_params:
            metadata.model_params = model_params

        return metadata
