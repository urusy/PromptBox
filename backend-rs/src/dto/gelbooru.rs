//! Gelbooru tag search DTOs (mirror schemas/gelbooru.py).

#[derive(Debug, Clone, serde::Serialize)]
pub struct GelbooruTag {
    pub id: i64,
    pub name: String,
    pub count: i64,
    /// 0=general, 1=artist, 3=copyright, 4=character, 5=metadata.
    pub r#type: i64,
    pub ambiguous: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GelbooruTagSearchResponse {
    pub tags: Vec<GelbooruTag>,
    pub query: String,
}
