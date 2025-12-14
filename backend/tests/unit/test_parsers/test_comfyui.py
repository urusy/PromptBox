"""Tests for ComfyUI parser."""

import json
from typing import Any

import pytest

from app.parsers.base import SourceTool
from app.parsers.comfyui import ComfyUIParser


class TestComfyUIParser:
    """Test cases for ComfyUIParser."""

    @pytest.fixture
    def parser(self) -> ComfyUIParser:
        """Create a parser instance."""
        return ComfyUIParser()

    def test_can_parse_valid_comfyui(
        self, parser: ComfyUIParser, sample_png_info_comfyui: dict[str, Any]
    ) -> None:
        """Test that parser recognizes valid ComfyUI metadata."""
        assert parser.can_parse(sample_png_info_comfyui) is True

    def test_can_parse_invalid_no_prompt(self, parser: ComfyUIParser) -> None:
        """Test that parser rejects metadata without prompt key."""
        png_info = {"other": "data"}
        assert parser.can_parse(png_info) is False

    def test_can_parse_invalid_json(self, parser: ComfyUIParser) -> None:
        """Test that parser rejects invalid JSON."""
        png_info = {"prompt": "not valid json {{{"}
        assert parser.can_parse(png_info) is False

    def test_can_parse_invalid_no_class_type(self, parser: ComfyUIParser) -> None:
        """Test that parser rejects nodes without class_type."""
        png_info = {"prompt": '{"1": {"inputs": {}}}'}
        assert parser.can_parse(png_info) is False

    def test_parse_basic_metadata(
        self, parser: ComfyUIParser, sample_png_info_comfyui: dict[str, Any]
    ) -> None:
        """Test parsing basic ComfyUI metadata."""
        result = parser.parse(sample_png_info_comfyui)

        assert result.source_tool == SourceTool.COMFYUI
        assert result.has_metadata is True
        assert result.seed == 12345
        assert result.steps == 20
        assert result.cfg_scale == 7.5
        assert result.sampler_name == "euler"
        assert result.scheduler == "normal"

    def test_parse_prompt_extraction(
        self, parser: ComfyUIParser, sample_png_info_comfyui: dict[str, Any]
    ) -> None:
        """Test prompt extraction from CLIPTextEncode nodes."""
        result = parser.parse(sample_png_info_comfyui)

        assert result.positive_prompt is not None
        assert "sunset" in result.positive_prompt.lower()
        assert result.negative_prompt is not None
        assert "ugly" in result.negative_prompt.lower()

    def test_parse_checkpoint_name(
        self, parser: ComfyUIParser, sample_png_info_comfyui: dict[str, Any]
    ) -> None:
        """Test checkpoint name extraction."""
        result = parser.parse(sample_png_info_comfyui)

        assert result.model_name == "sd_xl_base_1.0"

    def test_parse_lora_extraction(self, parser: ComfyUIParser) -> None:
        """Test LoRA extraction from workflow."""
        png_info = {
            "prompt": json.dumps({
                "1": {
                    "class_type": "LoraLoader",
                    "inputs": {
                        "lora_name": "detail_slider.safetensors",
                        "strength_model": 0.8,
                        "strength_clip": 0.5,
                    },
                },
                "2": {
                    "class_type": "KSampler",
                    "inputs": {
                        "seed": 1,
                        "steps": 20,
                        "cfg": 7,
                        "sampler_name": "euler",
                        "scheduler": "normal",
                    },
                },
            })
        }
        result = parser.parse(png_info)

        assert len(result.loras) == 1
        assert result.loras[0].name == "detail_slider"
        assert result.loras[0].weight == 0.8
        assert result.loras[0].weight_clip == 0.5

    def test_parse_controlnet_extraction(self, parser: ComfyUIParser) -> None:
        """Test ControlNet extraction from workflow."""
        png_info = {
            "prompt": json.dumps({
                "1": {
                    "class_type": "ControlNetLoader",
                    "inputs": {"control_net_name": "canny.safetensors"},
                },
                "2": {
                    "class_type": "ControlNetApplyAdvanced",
                    "inputs": {
                        "control_net": ["1", 0],
                        "strength": 0.7,
                        "start_percent": 0.0,
                        "end_percent": 0.8,
                    },
                },
                "3": {
                    "class_type": "KSampler",
                    "inputs": {
                        "seed": 1,
                        "steps": 20,
                        "cfg": 7,
                        "sampler_name": "euler",
                        "scheduler": "normal",
                    },
                },
            })
        }
        result = parser.parse(png_info)

        assert len(result.controlnets) == 1
        assert result.controlnets[0].model == "canny.safetensors"
        assert result.controlnets[0].weight == 0.7
        assert result.controlnets[0].guidance_end == 0.8

    def test_parse_upscale_info(self, parser: ComfyUIParser) -> None:
        """Test upscale information extraction."""
        png_info = {
            "prompt": json.dumps({
                "1": {
                    "class_type": "LatentUpscaleBy",
                    "inputs": {
                        "upscale_method": "bilinear",
                        "scale_by": 2.0,
                    },
                },
                "2": {
                    "class_type": "KSampler",
                    "inputs": {
                        "seed": 1,
                        "steps": 20,
                        "cfg": 7,
                        "sampler_name": "euler",
                        "scheduler": "normal",
                    },
                },
            })
        }
        result = parser.parse(png_info)

        assert result.model_params.get("hires_upscale") == 2.0
        assert result.model_params.get("upscale_method") == "latent"

    def test_parse_workflow_extras(
        self, parser: ComfyUIParser, sample_png_info_comfyui: dict[str, Any]
    ) -> None:
        """Test workflow extras extraction."""
        result = parser.parse(sample_png_info_comfyui)

        assert result.workflow_extras.get("node_count") == 4
        assert result.workflow_extras.get("workflow_version") == "0.1.0"

    def test_parse_dict_prompt(self, parser: ComfyUIParser) -> None:
        """Test parsing when prompt is already a dict (not string)."""
        png_info = {
            "prompt": {
                "1": {
                    "class_type": "KSampler",
                    "inputs": {
                        "seed": 999,
                        "steps": 25,
                        "cfg": 8,
                        "sampler_name": "dpmpp_2m",
                        "scheduler": "karras",
                    },
                }
            }
        }
        result = parser.parse(png_info)

        assert result.seed == 999
        assert result.steps == 25
        assert result.sampler_name == "dpmpp_2m"
