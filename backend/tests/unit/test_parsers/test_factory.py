"""Tests for MetadataParserFactory."""

from typing import Any

import pytest

from app.parsers.base import ParsedMetadata, SourceTool
from app.parsers.factory import MetadataParserFactory


class TestMetadataParserFactory:
    """Test cases for MetadataParserFactory."""

    @pytest.fixture
    def factory(self) -> MetadataParserFactory:
        """Create a parser factory instance."""
        return MetadataParserFactory()

    def test_parse_comfyui_metadata(
        self, factory: MetadataParserFactory, sample_png_info_comfyui: dict[str, Any]
    ) -> None:
        """Test parsing ComfyUI metadata."""
        result = factory.parse(sample_png_info_comfyui)

        assert isinstance(result, ParsedMetadata)
        assert result.source_tool == SourceTool.COMFYUI
        assert result.has_metadata is True
        assert result.seed == 12345
        assert result.steps == 20
        assert result.cfg_scale == 7.5
        assert result.sampler_name == "euler"
        assert result.scheduler == "normal"
        assert result.positive_prompt is not None
        assert "sunset" in result.positive_prompt.lower()

    def test_parse_a1111_metadata(
        self, factory: MetadataParserFactory, sample_png_info_a1111: dict[str, Any]
    ) -> None:
        """Test parsing A1111 metadata."""
        result = factory.parse(sample_png_info_a1111)

        assert isinstance(result, ParsedMetadata)
        assert result.source_tool == SourceTool.A1111
        assert result.has_metadata is True
        assert result.seed == 67890
        assert result.steps == 30
        assert result.cfg_scale == 7.5
        assert result.sampler_name is not None
        assert "DPM" in result.sampler_name
        assert result.positive_prompt is not None
        assert "sunset" in result.positive_prompt.lower()
        assert len(result.loras) >= 1
        assert result.loras[0].name == "detail_slider"
        assert result.loras[0].weight == 0.8

    def test_parse_novelai_metadata(
        self, factory: MetadataParserFactory, sample_png_info_novelai: dict[str, Any]
    ) -> None:
        """Test parsing NovelAI metadata."""
        result = factory.parse(sample_png_info_novelai)

        assert isinstance(result, ParsedMetadata)
        assert result.source_tool == SourceTool.NOVELAI
        assert result.has_metadata is True
        assert result.seed == 111222
        assert result.steps == 28
        assert result.cfg_scale == 11.0
        assert result.positive_prompt is not None
        assert "sunset" in result.positive_prompt.lower()

    def test_parse_empty_metadata(
        self, factory: MetadataParserFactory, sample_png_info_empty: dict[str, Any]
    ) -> None:
        """Test parsing empty/unknown metadata."""
        result = factory.parse(sample_png_info_empty)

        assert isinstance(result, ParsedMetadata)
        assert result.source_tool == SourceTool.UNKNOWN
        assert result.has_metadata is False

    def test_parse_invalid_json_metadata(self, factory: MetadataParserFactory) -> None:
        """Test parsing invalid JSON in metadata."""
        invalid_metadata = {"prompt": "not valid json {{{"}
        result = factory.parse(invalid_metadata)

        assert isinstance(result, ParsedMetadata)
        assert result.source_tool == SourceTool.UNKNOWN
        assert result.has_metadata is False

    def test_factory_parser_priority(self, factory: MetadataParserFactory) -> None:
        """Test that factory has correct parser priority."""
        # ComfyUIParser should come first
        from app.parsers.comfyui import ComfyUIParser

        assert isinstance(factory.parsers[0], ComfyUIParser)
