"""Tests for NovelAI parser."""

import json
from typing import Any

import pytest

from app.parsers.base import ModelType, SourceTool
from app.parsers.novelai import NovelAIParser


class TestNovelAIParser:
    """Test cases for NovelAIParser."""

    @pytest.fixture
    def parser(self) -> NovelAIParser:
        """Create a parser instance."""
        return NovelAIParser()

    def test_can_parse_valid_novelai(
        self, parser: NovelAIParser, sample_png_info_novelai: dict[str, Any]
    ) -> None:
        """Test that parser recognizes valid NovelAI metadata."""
        assert parser.can_parse(sample_png_info_novelai) is True

    def test_can_parse_invalid_no_comment(self, parser: NovelAIParser) -> None:
        """Test that parser rejects metadata without Comment key."""
        png_info = {"other": "data"}
        assert parser.can_parse(png_info) is False

    def test_can_parse_invalid_no_prompt_or_uc(self, parser: NovelAIParser) -> None:
        """Test that parser rejects metadata without prompt or uc."""
        png_info = {"Comment": '{"other": "data"}'}
        assert parser.can_parse(png_info) is False

    def test_can_parse_invalid_json(self, parser: NovelAIParser) -> None:
        """Test that parser rejects invalid JSON."""
        png_info = {"Comment": "not valid json {{{"}
        assert parser.can_parse(png_info) is False

    def test_parse_basic_metadata(
        self, parser: NovelAIParser, sample_png_info_novelai: dict[str, Any]
    ) -> None:
        """Test parsing basic NovelAI metadata."""
        result = parser.parse(sample_png_info_novelai)

        assert result.source_tool == SourceTool.NOVELAI
        assert result.has_metadata is True
        assert result.model_type == ModelType.OTHER
        assert result.seed == 111222
        assert result.steps == 28
        assert result.cfg_scale == 11.0

    def test_parse_prompts(
        self, parser: NovelAIParser, sample_png_info_novelai: dict[str, Any]
    ) -> None:
        """Test prompt extraction."""
        result = parser.parse(sample_png_info_novelai)

        assert result.positive_prompt is not None
        assert "sunset" in result.positive_prompt.lower()
        assert result.negative_prompt is not None
        assert "ugly" in result.negative_prompt.lower()

    def test_parse_sampler_name_strip_prefix(self, parser: NovelAIParser) -> None:
        """Test that k_ prefix is stripped from sampler name."""
        png_info = {
            "Comment": json.dumps({
                "prompt": "test",
                "sampler": "k_euler_ancestral",
            })
        }
        result = parser.parse(png_info)

        assert result.sampler_name == "euler_ancestral"

    def test_parse_sampler_without_prefix(self, parser: NovelAIParser) -> None:
        """Test sampler name without k_ prefix."""
        png_info = {
            "Comment": json.dumps({
                "prompt": "test",
                "sampler": "ddim",
            })
        }
        result = parser.parse(png_info)

        assert result.sampler_name == "ddim"

    def test_parse_additional_params(self, parser: NovelAIParser) -> None:
        """Test extraction of additional NovelAI parameters."""
        png_info = {
            "Comment": json.dumps({
                "prompt": "test",
                "width": 1024,
                "height": 1536,
                "n_samples": 4,
                "ucPreset": 2,
                "qualityToggle": True,
            })
        }
        result = parser.parse(png_info)

        assert result.model_params.get("width") == 1024
        assert result.model_params.get("height") == 1536
        assert result.model_params.get("n_samples") == 4
        assert result.model_params.get("ucPreset") == 2
        assert result.model_params.get("qualityToggle") is True

    def test_parse_dict_comment(self, parser: NovelAIParser) -> None:
        """Test parsing when Comment is already a dict (not string)."""
        png_info = {
            "Comment": {
                "prompt": "already parsed",
                "uc": "bad",
                "steps": 30,
            }
        }
        result = parser.parse(png_info)

        assert result.positive_prompt == "already parsed"
        assert result.steps == 30

    def test_parse_invalid_numeric_values(self, parser: NovelAIParser) -> None:
        """Test handling of invalid numeric values."""
        png_info = {
            "Comment": json.dumps({
                "prompt": "test",
                "steps": "not a number",
                "scale": "invalid",
                "seed": "abc",
            })
        }
        result = parser.parse(png_info)

        assert result.steps is None
        assert result.cfg_scale is None
        assert result.seed is None
