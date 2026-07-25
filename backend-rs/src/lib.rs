//! PromptBox backend as a library.
//!
//! The binary (`src/main.rs`) is a thin wrapper around this crate. Everything
//! lives here so that integration tests under `tests/` can drive the real
//! store/HTTP code against a throwaway database (`#[sqlx::test]`), which is
//! impossible for modules private to a binary target.

pub mod auth;
pub mod batch;
pub mod cache;
pub mod catalog;
pub mod change;
pub mod civitai;
pub mod config;
pub mod db;
pub mod dto;
pub mod duplicate;
pub mod error;
pub mod export;
pub mod gelbooru;
pub mod http;
pub mod image;
pub mod job;
pub mod media;
pub mod parser;
pub mod preset;
pub mod showcase;
pub mod smart_folder;
pub mod stats;
pub mod storage;
pub mod tag;
pub mod util;
pub mod worker;
