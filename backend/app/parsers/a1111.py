import re
from typing import Any

from app.parsers.base import (
    LoraInfo,
    MetadataParser,
    ParsedMetadata,
    SourceTool,
)
from app.parsers.model_detector import detect_model_type


class A1111Parser(MetadataParser):
    """Parser for A1111/Forge metadata format."""

    # Regex for LoRA tags: <lora:name:weight> or <lora:name:weight:clip_weight>
    LORA_PATTERN = re.compile(r"<lora:([^:>]+):([0-9.]+)(?::([0-9.]+))?>")

    def can_parse(self, png_info: dict[str, Any]) -> bool:
        """Check if the PNG info contains A1111/Forge metadata."""
        if "parameters" not in png_info:
            return False

        params = png_info["parameters"]
        if not isinstance(params, str):
            return False

        # Check for characteristic patterns
        return "Steps:" in params and "Sampler:" in params

    def parse(self, png_info: dict[str, Any]) -> ParsedMetadata:
        """Parse A1111/Forge metadata."""
        params_str = png_info.get("parameters", "")

        # Detect if it's Forge
        source_tool = SourceTool.A1111
        if "Forge" in params_str:
            source_tool = SourceTool.FORGE

        metadata = ParsedMetadata(
            source_tool=source_tool,
            has_metadata=True,
            raw_metadata={"parameters": params_str},
        )

        # Parse the parameters string
        self._parse_parameters(params_str, metadata)

        # Detect model type
        metadata.model_type = detect_model_type(metadata.model_name)

        return metadata

    def _parse_parameters(self, params_str: str, metadata: ParsedMetadata) -> None:
        """Parse the parameters string."""
        lines = params_str.strip().split("\n")

        # Find the line with "Steps:" - this separates prompts from params
        params_line_idx = -1
        for i, line in enumerate(lines):
            if line.startswith("Steps:"):
                params_line_idx = i
                break

        if params_line_idx == -1:
            return

        # Everything before the params line is prompts
        prompt_lines = lines[:params_line_idx]
        params_line = lines[params_line_idx]

        # Parse prompts
        self._parse_prompts(prompt_lines, metadata)

        # Parse parameters line
        self._parse_params_line(params_line, metadata)

        # Extract LoRAs from positive prompt
        if metadata.positive_prompt:
            self._extract_loras(metadata.positive_prompt, metadata)

    def _parse_prompts(self, lines: list[str], metadata: ParsedMetadata) -> None:
        """Parse positive and negative prompts from lines."""
        if not lines:
            return

        prompt_parts: list[str] = []
        negative_parts: list[str] = []
        in_negative = False

        for line in lines:
            if line.startswith("Negative prompt:"):
                in_negative = True
                # Get the part after "Negative prompt:"
                neg_text = line[len("Negative prompt:"):].strip()
                if neg_text:
                    negative_parts.append(neg_text)
            elif in_negative:
                negative_parts.append(line)
            else:
                prompt_parts.append(line)

        positive = " ".join(prompt_parts).strip()
        negative = " ".join(negative_parts).strip()

        if positive:
            metadata.positive_prompt = positive
        if negative:
            metadata.negative_prompt = negative

    def _parse_params_line(self, params_line: str, metadata: ParsedMetadata) -> None:
        """Parse the parameters line (key: value, key: value, ...)."""
        # Split by comma, but be careful with values that contain commas
        parts = self._split_params(params_line)

        model_params: dict[str, Any] = {}

        for part in parts:
            if ":" not in part:
                continue

            key, value = part.split(":", 1)
            key = key.strip()
            value = value.strip()

            # Map to metadata fields
            if key == "Steps":
                try:
                    metadata.steps = int(value)
                except ValueError:
                    pass
            elif key == "Sampler":
                metadata.sampler_name = value
            elif key == "CFG scale":
                try:
                    metadata.cfg_scale = float(value)
                except ValueError:
                    pass
            elif key == "Seed":
                try:
                    metadata.seed = int(value)
                except ValueError:
                    pass
            elif key == "Model":
                metadata.model_name = value
            elif key == "Scheduler":
                metadata.scheduler = value
            elif key == "Clip skip":
                try:
                    model_params["clip_skip"] = int(value)
                except ValueError:
                    pass
            elif key == "VAE":
                model_params["vae"] = value
            elif key == "Model hash":
                model_params["model_hash"] = value
            elif key == "Size":
                model_params["size"] = value
            # Hires upscaler parameters
            elif key == "Hires upscale":
                try:
                    model_params["hires_upscale"] = float(value)
                except ValueError:
                    pass
            elif key == "Hires upscaler":
                model_params["hires_upscaler"] = value
            elif key == "Hires steps":
                try:
                    model_params["hires_steps"] = int(value)
                except ValueError:
                    pass
            elif key == "Denoising strength":
                try:
                    model_params["denoising_strength"] = float(value)
                except ValueError:
                    pass
            # XYZ grid parameters
            elif key == "X/Y/Z plot":
                model_params["is_xyz_grid"] = True
            elif key == "X Type":
                model_params["xyz_x_type"] = value
            elif key == "X Values":
                model_params["xyz_x_values"] = value.strip('"')
            elif key == "Y Type":
                model_params["xyz_y_type"] = value
            elif key == "Y Values":
                model_params["xyz_y_values"] = value.strip('"')
            elif key == "Z Type":
                model_params["xyz_z_type"] = value
            elif key == "Z Values":
                model_params["xyz_z_values"] = value.strip('"')

        if model_params:
            metadata.model_params = model_params

    def _split_params(self, params_line: str) -> list[str]:
        """Split parameters line handling quoted values."""
        parts: list[str] = []
        current = ""
        in_quotes = False

        for char in params_line:
            if char == '"':
                in_quotes = not in_quotes
                current += char
            elif char == "," and not in_quotes:
                if current.strip():
                    parts.append(current.strip())
                current = ""
            else:
                current += char

        if current.strip():
            parts.append(current.strip())

        return parts

    def _extract_loras(self, prompt: str, metadata: ParsedMetadata) -> None:
        """Extract LoRA information from prompt text."""
        matches = self.LORA_PATTERN.findall(prompt)

        for match in matches:
            name = match[0]
            weight = float(match[1]) if match[1] else 1.0
            weight_clip = float(match[2]) if match[2] else None

            lora_info = LoraInfo(
                name=name,
                weight=weight,
                weight_clip=weight_clip,
            )
            metadata.loras.append(lora_info)
