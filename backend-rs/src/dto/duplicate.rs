//! Duplicate-file management DTOs (mirror endpoints/duplicates.py).

#[derive(Debug, serde::Serialize)]
pub struct DuplicatesInfo {
    pub count: i64,
    pub total_size_bytes: i64,
    pub files: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct DeleteResult {
    pub deleted_count: i64,
    pub freed_bytes: i64,
}

#[derive(Debug, serde::Serialize)]
pub struct DeleteFileResult {
    pub deleted: String,
    pub freed_bytes: i64,
}
