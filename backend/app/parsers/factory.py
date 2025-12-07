from typing import Any

from loguru import logger

from app.parsers.base import MetadataParser, ParsedMetadata, SourceTool
from app.parsers.comfyui import ComfyUIParser
from app.parsers.a1111 import A1111Parser
from app.parsers.novelai import NovelAIParser


class MetadataParserFactory:
    """Factory for selecting and running the appropriate metadata parser."""

    def __init__(self) -> None:
        # Parsers in priority order
        self.parsers: list[MetadataParser] = [
            ComfyUIParser(),
            A1111Parser(),
            NovelAIParser(),
        ]

    def parse(self, png_info: dict[str, Any]) -> ParsedMetadata:
        """Select the appropriate parser and parse the metadata.

        Args:
            png_info: Dictionary of PNG metadata

        Returns:
            ParsedMetadata object with extracted information
        """
        for parser in self.parsers:
            if parser.can_parse(png_info):
                try:
                    return parser.parse(png_info)
                except Exception as e:
                    logger.warning(f"Parse error with {parser.__class__.__name__}: {e}")
                    return ParsedMetadata(
                        source_tool=SourceTool.UNKNOWN,
                        has_metadata=False,
                        workflow_extras={"parse_error": str(e)},
                    )

        # No parser matched
        return ParsedMetadata(
            source_tool=SourceTool.UNKNOWN,
            has_metadata=False,
        )
