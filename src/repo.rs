use crate::models::{MediaItem, Query};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Other: {0}")]
    Other(String),
}

pub type RepoResult<T> = Result<T, RepoError>;

pub trait Repository: Send + Sync {
    fn init(&self) -> RepoResult<()>;
    fn add(&self, item: &mut MediaItem) -> RepoResult<i64>;
    fn update(&self, item: &MediaItem) -> RepoResult<()>;
    fn delete(&self, id: i64) -> RepoResult<()>;
    fn get(&self, id: i64) -> RepoResult<Option<MediaItem>>;
    fn list(&self, query: &Query) -> RepoResult<Vec<MediaItem>>;
    fn stats(&self) -> RepoResult<Stats>;
}

#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub total: usize,
    pub by_category: Vec<(String, usize)>,
    pub finished: usize,
    pub unfinished: usize,
}
