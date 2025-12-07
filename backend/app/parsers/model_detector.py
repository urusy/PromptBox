from app.parsers.base import ModelType


def detect_model_type(model_name: str | None) -> ModelType:
    """Detect model type from model name."""
    if not model_name:
        return ModelType.OTHER

    name_lower = model_name.lower()

    # Detection rules (priority order)
    rules: list[tuple[list[str], ModelType]] = [
        (["qwen"], ModelType.QWEN),
        (["flux"], ModelType.FLUX),
        (["pony"], ModelType.PONY),
        (["illustrious", "noob"], ModelType.ILLUSTRIOUS),
        (["xl", "sdxl"], ModelType.SDXL),
        (["sd15", "v1-5", "1.5", "sd_1"], ModelType.SD15),
    ]

    for keywords, model_type in rules:
        if any(kw in name_lower for kw in keywords):
            return model_type

    return ModelType.OTHER
