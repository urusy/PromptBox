"""Gelbooru tag search schemas."""

from pydantic import BaseModel


class GelbooruTag(BaseModel):
    id: int
    name: str
    count: int
    type: int  # 0=general, 1=artist, 3=copyright, 4=character, 5=metadata
    ambiguous: bool = False


class GelbooruTagSearchResponse(BaseModel):
    tags: list[GelbooruTag]
    query: str
