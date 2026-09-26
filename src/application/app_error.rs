use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),
}

pub type AppResult<T> = Result<T, AppError>;
