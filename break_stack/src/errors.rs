use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("user is not authenticated")]
    Unauthenticated,
    #[error("user is not authorized")]
    Unauthorized,
    #[error("model error")]
    Model(#[from] ModelError),
}

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("resource not found")]
    NotFound,
    #[error("operation is not allowed because of a conflict")]
    Conflict,
    #[error("database error")]
    DB(sqlx::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for ModelError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ModelError::NotFound,
            sqlx::Error::Database(ref db_err) => match db_err.kind() {
                sqlx::error::ErrorKind::UniqueViolation
                | sqlx::error::ErrorKind::ForeignKeyViolation
                | sqlx::error::ErrorKind::NotNullViolation
                | sqlx::error::ErrorKind::CheckViolation => ModelError::Conflict,
                _ => ModelError::DB(err),
            },
            _ => ModelError::DB(err),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("model error")]
    Model(#[from] ModelError),
    #[error("resource not found")]
    NotFound,
    #[error("authentication/authorization error")]
    Auth(#[from] AuthError),
    #[error("failed to log in")]
    Login,
    #[error("internal error: {0}")]
    Internal(String),
    #[error("bad request: {0}")]
    BadRequest(String),
}
