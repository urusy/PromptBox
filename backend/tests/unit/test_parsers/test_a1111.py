"""Tests for A1111 parser."""

from typing import Any

import pytest

from app.parsers.a1111 import A1111Parser
from app.parsers.base import SourceTool


class TestA1111Parser:
    """Test cases for A1111Parser."""

    @pytest.fixture
    def parser(self) -> A1111Parser:
        """Create a parser instance."""
        return A1111Parser()

    def test_can_parse_valid_a1111(self, parser: A1111Parser) -> None:
        """Test that parser recognizes valid A1111 metadata."""
        png_info = {
            "parameters": "test prompt\nSteps: 20, Sampler: Euler, CFG scale: 7"
        }
        assert parser.can_parse(png_info) is True

    def test_can_parse_invalid_no_parameters(self, parser: A1111Parser) -> None:
        """Test that parser rejects metadata without parameters key."""
        png_info = {"other": "data"}
        assert parser.can_parse(png_info) is False

    def test_can_parse_invalid_no_steps(self, parser: A1111Parser) -> None:
        """Test that parser rejects metadata without Steps."""
        png_info = {"parameters": "test prompt without steps info"}
        assert parser.can_parse(png_info) is False

    def test_parse_basic_metadata(
        self, parser: A1111Parser, sample_png_info_a1111: dict[str, Any]
    ) -> None:
        """Test parsing basic A1111 metadata."""
        result = parser.parse(sample_png_info_a1111)

        assert result.source_tool == SourceTool.A1111
        assert result.has_metadata is True
        assert result.positive_prompt is not None
        assert "sunset" in result.positive_prompt.lower()
        assert result.negative_prompt == "ugly, blurry"
        assert result.steps == 30
        assert result.sampler_name == "DPM++ 2M Karras"
        assert result.cfg_scale == 7.5
        assert result.seed == 67890

    def test_parse_forge_detection(self, parser: A1111Parser) -> None:
        """Test that Forge is detected correctly."""
        png_info = {
            "parameters": "test prompt\nSteps: 20, Sampler: Euler, Forge Version: 1.0"
        }
        result = parser.parse(png_info)
        assert result.source_tool == SourceTool.FORGE

    def test_parse_lora_extraction(self, parser: A1111Parser) -> None:
        """Test LoRA extraction from prompt."""
        png_info = {
            "parameters": """photo, <lora:add_detail:0.8>, <lora:skin_texture:0.5:0.3>
Negative prompt: bad
Steps: 20, Sampler: Euler, CFG scale: 7, Seed: 12345, Model: test"""
        }
        result = parser.parse(png_info)

        assert len(result.loras) == 2
        assert result.loras[0].name == "add_detail"
        assert result.loras[0].weight == 0.8
        assert result.loras[1].name == "skin_texture"
        assert result.loras[1].weight == 0.5
        assert result.loras[1].weight_clip == 0.3

    def test_parse_hires_parameters(self, parser: A1111Parser) -> None:
        """Test parsing of hires/upscale parameters."""
        png_info = {
            "parameters": """test prompt
Steps: 20, Sampler: Euler, CFG scale: 7, Seed: 12345, Hires upscale: 2.0, Hires upscaler: 4x-UltraSharp, Hires steps: 15, Denoising strength: 0.5"""
        }
        result = parser.parse(png_info)

        assert result.model_params.get("hires_upscale") == 2.0
        assert result.model_params.get("hires_upscaler") == "4x-UltraSharp"
        assert result.model_params.get("hires_steps") == 15
        assert result.model_params.get("denoising_strength") == 0.5

    def test_parse_xyz_grid_parameters(self, parser: A1111Parser) -> None:
        """Test parsing of XYZ grid parameters."""
        png_info = {
            "parameters": """test prompt
Steps: 20, Sampler: Euler, CFG scale: 7, Seed: 12345, Script: X/Y/Z plot, X Type: CFG Scale, X Values: "5, 7, 9", Y Type: Steps, Y Values: "20, 30\""""
        }
        result = parser.parse(png_info)

        assert result.model_params.get("is_xyz_grid") is True
        assert result.model_params.get("xyz_x_type") == "CFG Scale"
        assert result.model_params.get("xyz_x_values") == "5, 7, 9"

    def test_parse_multiline_prompt(self, parser: A1111Parser) -> None:
        """Test parsing of multiline prompts."""
        png_info = {
            "parameters": """first line of prompt,
second line of prompt,
third line
Negative prompt: bad,
ugly,
blurry
Steps: 20, Sampler: Euler, CFG scale: 7, Seed: 12345"""
        }
        result = parser.parse(png_info)

        assert "first line" in result.positive_prompt
        assert "third line" in result.positive_prompt
        assert "bad" in result.negative_prompt
        assert "blurry" in result.negative_prompt
