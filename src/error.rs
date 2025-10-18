use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("version control error: {0}")]
    VersionControl(String),
    #[error("issue tracker error: {0}")]
    IssueTracker(String),
    #[error("language model error: {0}")]
    LanguageModel(String),
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub type AppResult<T> = Result<T, AppError>;
