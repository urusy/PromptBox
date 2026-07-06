//! Smart folder DTOs (mirror schemas/smart_folder.py). Shares `SearchFilters`
//! with search presets.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::dto::preset::SearchFilters;

#[derive(Debug, serde::Deserialize)]
pub struct SmartFolderCreate {
    pub name: String,
    pub icon: Option<String>,
    pub filters: SearchFilters,
}

#[derive(Debug, serde::Deserialize)]
pub struct SmartFolderUpdate {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub filters: Option<SearchFilters>,
}

#[derive(Debug, serde::Serialize)]
pub struct SmartFolderResponse {
    pub id: Uuid,
    pub name: String,
    pub icon: Option<String>,
    pub filters: SearchFilters,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
