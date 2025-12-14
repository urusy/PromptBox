import csv
import io
import json
from uuid import UUID

from fastapi import APIRouter, Query
from fastapi.responses import StreamingResponse
from pydantic import BaseModel, Field

from app.api.deps import CurrentUser, DbSession
from app.services.export_service import ExportService

router = APIRouter(prefix="/export", tags=["export"])


class ExportRequest(BaseModel):
    ids: list[UUID] | None = None
    export_format: str = Field("json", pattern="^(json|csv)$")


@router.get("/metadata")
async def export_metadata(
    db: DbSession,
    _: CurrentUser,
    ids: list[UUID] | None = Query(None),
    export_format: str = Query("json", pattern="^(json|csv)$"),
) -> StreamingResponse:
    """Export image metadata as JSON or CSV."""
    service = ExportService(db)
    data = await service.get_export_data(ids)

    if export_format == "csv":
        output = io.StringIO()
        if data:
            writer = csv.DictWriter(output, fieldnames=data[0].keys())
            writer.writeheader()
            writer.writerows(data)
        content = output.getvalue()
        media_type = "text/csv"
        filename = "comfyui_gallery_export.csv"
    else:
        content = json.dumps(data, indent=2, default=str)
        media_type = "application/json"
        filename = "comfyui_gallery_export.json"

    return StreamingResponse(
        iter([content]),
        media_type=media_type,
        headers={"Content-Disposition": f"attachment; filename={filename}"},
    )


@router.get("/prompts")
async def export_prompts(
    db: DbSession,
    _: CurrentUser,
    ids: list[UUID] | None = Query(None),
) -> StreamingResponse:
    """Export prompts as a text file."""
    service = ExportService(db)
    content = await service.get_prompts_export(ids)

    return StreamingResponse(
        iter([content]),
        media_type="text/plain",
        headers={"Content-Disposition": "attachment; filename=prompts.txt"},
    )
