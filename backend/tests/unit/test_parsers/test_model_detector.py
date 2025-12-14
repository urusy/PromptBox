"""Tests for model type detection."""

import pytest

from app.parsers.base import ModelType
from app.parsers.model_detector import detect_model_type


class TestModelDetector:
    """Test cases for model type detection."""

    @pytest.mark.parametrize(
        "model_name,expected",
        [
            # SDXL models
            ("sd_xl_base_1.0", ModelType.SDXL),
            ("animagineXL_v30", ModelType.SDXL),
            ("sdxl_turbo", ModelType.SDXL),
            # SD 1.5 models
            ("v1-5-pruned-emaonly", ModelType.SD15),
            ("sd_1.5_model", ModelType.SD15),
            ("dreamshaper_sd15", ModelType.SD15),
            # Pony models
            ("ponyDiffusionV6XL", ModelType.PONY),
            ("autismmixPony_v10", ModelType.PONY),
            # Illustrious models
            ("illustriousXL_v01", ModelType.ILLUSTRIOUS),
            ("noobaiXL_v1", ModelType.ILLUSTRIOUS),
            # FLUX models
            ("flux1-dev", ModelType.FLUX),
            ("flux-schnell", ModelType.FLUX),
            # Qwen models
            ("qwen2-vl-7b", ModelType.QWEN),
            # Unknown/Other
            ("random_model_name", ModelType.OTHER),
            ("", ModelType.OTHER),
            (None, ModelType.OTHER),
        ],
    )
    def test_detect_model_type(
        self, model_name: str | None, expected: ModelType
    ) -> None:
        """Test model type detection for various model names."""
        result = detect_model_type(model_name)
        assert result == expected

    def test_detection_is_case_insensitive(self) -> None:
        """Test that model detection is case insensitive."""
        assert detect_model_type("SDXL_MODEL") == ModelType.SDXL
        assert detect_model_type("PonyDiffusion") == ModelType.PONY
        assert detect_model_type("FLUX-Dev") == ModelType.FLUX

    def test_detection_priority(self) -> None:
        """Test that detection respects priority order."""
        # Pony should take priority over XL
        assert detect_model_type("ponyXL") == ModelType.PONY
        # Illustrious/noob should take priority over XL
        assert detect_model_type("noobXL") == ModelType.ILLUSTRIOUS
