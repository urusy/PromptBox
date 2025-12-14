import json
from typing import Any, ClassVar

from app.parsers.base import (
    ControlNetInfo,
    LoraInfo,
    MetadataParser,
    ParsedMetadata,
    SourceTool,
)
from app.parsers.model_detector import detect_model_type


class ComfyUIParser(MetadataParser):
    """Parser for ComfyUI metadata format."""

    # Node types to extract data from
    SAMPLER_NODES: ClassVar[set[str]] = {
        "KSampler",
        "KSamplerAdvanced",
        "SamplerCustom",
    }
    CHECKPOINT_NODES: ClassVar[set[str]] = {
        "CheckpointLoaderSimple",
        "CheckpointLoader",
        "UNETLoader",
    }
    PROMPT_NODES: ClassVar[set[str]] = {"CLIPTextEncode", "CLIPTextEncodeSDXL"}
    LORA_NODES: ClassVar[set[str]] = {"LoraLoader", "LoraLoaderModelOnly"}
    CONTROLNET_NODES: ClassVar[set[str]] = {
        "ControlNetLoader",
        "ControlNetApply",
        "ControlNetApplyAdvanced",
    }
    UPSCALE_MODEL_NODES: ClassVar[set[str]] = {"UpscaleModelLoader"}
    UPSCALE_NODES: ClassVar[set[str]] = {
        "ImageUpscaleWithModel",
        "LatentUpscale",
        "LatentUpscaleBy",
        "ImageScaleBy",
    }

    def can_parse(self, png_info: dict[str, Any]) -> bool:
        """Check if the PNG info contains ComfyUI metadata."""
        if "prompt" not in png_info:
            return False

        try:
            prompt_data = png_info["prompt"]
            if isinstance(prompt_data, str):
                prompt_data = json.loads(prompt_data)

            # Check if it has the node structure
            if not isinstance(prompt_data, dict):
                return False

            # Check for at least one node with class_type
            for node in prompt_data.values():
                if isinstance(node, dict) and "class_type" in node:
                    return True

            return False
        except (json.JSONDecodeError, TypeError):
            return False

    def parse(self, png_info: dict[str, Any]) -> ParsedMetadata:
        """Parse ComfyUI metadata."""
        prompt_str = png_info.get("prompt", "{}")
        workflow_str = png_info.get("workflow", "{}")

        prompt_data = (
            json.loads(prompt_str) if isinstance(prompt_str, str) else prompt_str
        )
        workflow_data = (
            json.loads(workflow_str) if isinstance(workflow_str, str) else workflow_str
        )

        metadata = ParsedMetadata(
            source_tool=SourceTool.COMFYUI,
            has_metadata=True,
            raw_metadata={"prompt": prompt_data, "workflow": workflow_data},
        )

        # Extract data from nodes
        self._extract_sampler_data(prompt_data, metadata)
        self._extract_checkpoint_data(prompt_data, metadata)
        self._extract_prompts(prompt_data, metadata)
        self._extract_loras(prompt_data, metadata)
        self._extract_controlnets(prompt_data, metadata)
        self._extract_upscale_info(prompt_data, metadata)
        self._extract_workflow_extras(workflow_data, metadata)

        # Detect model type
        metadata.model_type = detect_model_type(metadata.model_name)

        return metadata

    def _extract_sampler_data(
        self, prompt_data: dict[str, Any], metadata: ParsedMetadata
    ) -> None:
        """Extract sampler parameters from KSampler nodes."""
        for node in prompt_data.values():
            if not isinstance(node, dict):
                continue

            class_type = node.get("class_type", "")
            if class_type not in self.SAMPLER_NODES:
                continue

            inputs = node.get("inputs", {})

            if "seed" in inputs:
                seed_val = inputs["seed"]
                if isinstance(seed_val, (int, float)):
                    metadata.seed = int(seed_val)

            if "steps" in inputs:
                steps_val = inputs["steps"]
                if isinstance(steps_val, (int, float)):
                    metadata.steps = int(steps_val)

            if "cfg" in inputs:
                cfg_val = inputs["cfg"]
                if isinstance(cfg_val, (int, float)):
                    metadata.cfg_scale = float(cfg_val)

            if "sampler_name" in inputs:
                metadata.sampler_name = str(inputs["sampler_name"])

            if "scheduler" in inputs:
                metadata.scheduler = str(inputs["scheduler"])

            # Found a sampler, stop looking
            break

    def _extract_checkpoint_data(
        self, prompt_data: dict[str, Any], metadata: ParsedMetadata
    ) -> None:
        """Extract checkpoint/model name."""
        for node in prompt_data.values():
            if not isinstance(node, dict):
                continue

            class_type = node.get("class_type", "")
            if class_type not in self.CHECKPOINT_NODES:
                continue

            inputs = node.get("inputs", {})

            ckpt_name = inputs.get("ckpt_name") or inputs.get("unet_name")
            if ckpt_name:
                # Remove file extension
                if isinstance(ckpt_name, str):
                    metadata.model_name = ckpt_name.rsplit(".", 1)[0]
                break

    def _extract_prompts(
        self, prompt_data: dict[str, Any], metadata: ParsedMetadata
    ) -> None:
        """Extract positive and negative prompts by following node references."""
        # Find KSampler to trace prompt references
        sampler_node = None
        for node in prompt_data.values():
            if isinstance(node, dict) and node.get("class_type") in self.SAMPLER_NODES:
                sampler_node = node
                break

        if not sampler_node:
            return

        inputs = sampler_node.get("inputs", {})

        # Get positive prompt
        positive_ref = inputs.get("positive")
        if positive_ref:
            positive_text = self._resolve_prompt_reference(prompt_data, positive_ref)
            if positive_text:
                metadata.positive_prompt = positive_text

        # Get negative prompt
        negative_ref = inputs.get("negative")
        if negative_ref:
            negative_text = self._resolve_prompt_reference(prompt_data, negative_ref)
            if negative_text:
                metadata.negative_prompt = negative_text

    def _resolve_prompt_reference(
        self, prompt_data: dict[str, Any], ref: Any
    ) -> str | None:
        """Resolve a node reference to get the prompt text."""
        if not isinstance(ref, list) or len(ref) < 1:
            return None

        node_id = str(ref[0])
        node = prompt_data.get(node_id)

        if not isinstance(node, dict):
            return None

        class_type = node.get("class_type", "")

        # Direct text encode node
        if class_type in self.PROMPT_NODES:
            inputs = node.get("inputs", {})
            text = inputs.get("text")
            if isinstance(text, str):
                return text
            # SDXL uses text_g and text_l
            text_g = inputs.get("text_g")
            if isinstance(text_g, str):
                return text_g

        # Might be a conditioning node, try to follow the chain
        inputs = node.get("inputs", {})
        for key in ["conditioning", "clip"]:
            next_ref = inputs.get(key)
            if isinstance(next_ref, list):
                result = self._resolve_prompt_reference(prompt_data, next_ref)
                if result:
                    return result

        return None

    def _extract_loras(
        self, prompt_data: dict[str, Any], metadata: ParsedMetadata
    ) -> None:
        """Extract LoRA information."""
        for node in prompt_data.values():
            if not isinstance(node, dict):
                continue

            class_type = node.get("class_type", "")
            if class_type not in self.LORA_NODES:
                continue

            inputs = node.get("inputs", {})
            lora_name = inputs.get("lora_name")

            if lora_name:
                weight = inputs.get("strength_model", 1.0)
                weight_clip = inputs.get("strength_clip")

                if isinstance(lora_name, str):
                    lora_info = LoraInfo(
                        name=lora_name.rsplit(".", 1)[0],
                        weight=(
                            float(weight) if isinstance(weight, (int, float)) else 1.0
                        ),
                        weight_clip=(
                            float(weight_clip)
                            if isinstance(weight_clip, (int, float))
                            else None
                        ),
                    )
                    metadata.loras.append(lora_info)

    def _extract_controlnets(
        self, prompt_data: dict[str, Any], metadata: ParsedMetadata
    ) -> None:
        """Extract ControlNet information."""
        controlnet_models: dict[str, str] = {}

        # First pass: collect model names
        for node_id, node in prompt_data.items():
            if not isinstance(node, dict):
                continue

            if node.get("class_type") == "ControlNetLoader":
                inputs = node.get("inputs", {})
                model_name = inputs.get("control_net_name")
                if model_name:
                    controlnet_models[node_id] = model_name

        # Second pass: collect apply nodes with weights
        for node in prompt_data.values():
            if not isinstance(node, dict):
                continue

            class_type = node.get("class_type", "")
            if class_type not in {"ControlNetApply", "ControlNetApplyAdvanced"}:
                continue

            inputs = node.get("inputs", {})
            strength = inputs.get("strength", 1.0)

            # Try to find the model reference
            control_net_ref = inputs.get("control_net")
            model_name = "unknown"

            if isinstance(control_net_ref, list) and len(control_net_ref) >= 1:
                loader_id = str(control_net_ref[0])
                model_name = controlnet_models.get(loader_id, "unknown")

            cn_info = ControlNetInfo(
                model=model_name,
                weight=float(strength) if isinstance(strength, (int, float)) else 1.0,
                guidance_start=float(inputs.get("start_percent", 0.0)),
                guidance_end=float(inputs.get("end_percent", 1.0)),
            )
            metadata.controlnets.append(cn_info)

    def _extract_upscale_info(
        self, prompt_data: dict[str, Any], metadata: ParsedMetadata
    ) -> None:
        """Extract upscale/hires information."""
        upscale_models: dict[str, str] = {}

        # First pass: collect upscale model names
        for node_id, node in prompt_data.items():
            if not isinstance(node, dict):
                continue

            if node.get("class_type") in self.UPSCALE_MODEL_NODES:
                inputs = node.get("inputs", {})
                model_name = inputs.get("model_name")
                if model_name:
                    upscale_models[node_id] = model_name

        # Second pass: detect upscale usage
        for node in prompt_data.values():
            if not isinstance(node, dict):
                continue

            class_type = node.get("class_type", "")
            inputs = node.get("inputs", {})

            if class_type == "ImageUpscaleWithModel":
                # Find the model reference
                upscale_model_ref = inputs.get("upscale_model")
                model_name = "unknown"
                if isinstance(upscale_model_ref, list) and len(upscale_model_ref) >= 1:
                    loader_id = str(upscale_model_ref[0])
                    model_name = upscale_models.get(loader_id, "unknown")

                metadata.model_params["hires_upscaler"] = model_name
                metadata.model_params["upscale_method"] = "model"

            elif class_type == "LatentUpscale":
                method = inputs.get("upscale_method", "nearest-exact")
                width = inputs.get("width")
                height = inputs.get("height")
                metadata.model_params["hires_upscaler"] = f"Latent ({method})"
                metadata.model_params["upscale_method"] = "latent"
                if width and height:
                    metadata.model_params["upscale_size"] = f"{width}x{height}"

            elif class_type == "LatentUpscaleBy":
                method = inputs.get("upscale_method", "nearest-exact")
                scale = inputs.get("scale_by", 1.0)
                metadata.model_params["hires_upscaler"] = f"Latent ({method})"
                metadata.model_params["hires_upscale"] = (
                    float(scale) if isinstance(scale, (int, float)) else 1.0
                )
                metadata.model_params["upscale_method"] = "latent"

            elif class_type == "ImageScaleBy":
                method = inputs.get("upscale_method", "nearest-exact")
                scale = inputs.get("scale_by", 1.0)
                metadata.model_params["hires_upscaler"] = f"Image ({method})"
                metadata.model_params["hires_upscale"] = (
                    float(scale) if isinstance(scale, (int, float)) else 1.0
                )
                metadata.model_params["upscale_method"] = "image"

    def _extract_workflow_extras(
        self, workflow_data: dict[str, Any], metadata: ParsedMetadata
    ) -> None:
        """Extract additional workflow information."""
        if not workflow_data:
            return

        extras: dict[str, Any] = {}

        # Count nodes
        nodes = workflow_data.get("nodes", [])
        if isinstance(nodes, list):
            extras["node_count"] = len(nodes)

        # Get workflow version if available
        version = workflow_data.get("version")
        if version:
            extras["workflow_version"] = version

        if extras:
            metadata.workflow_extras = extras
