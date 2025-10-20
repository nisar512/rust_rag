use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Elasticsearch error: {0}")]
    Elasticsearch(#[from] elasticsearch::Error),

    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Unexpected error: {0}")]
    Other(String),
}

pub type AppResult<T> = Result<T, AppError>;
