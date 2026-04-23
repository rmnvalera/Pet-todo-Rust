use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
#[allow(unused)]
pub enum AppError {
    #[error(transparent)]
    Database(sqlx::Error),

    #[error("Already exists")]
    AlreadyExists,
    #[error("Not found")]
    NotFound,
    #[error("Invalid credentials")]
    Unauthorized,
    #[error("Internal server error")]
    InternalError,
    #[error("Missing credentials")]
    MissingToken,
    #[error("Invalid credentials")]
    InvalidToken,
    #[error("Invalid Uuid")]
    InvalidId,
    #[error("Accsess denide!")]
    AccessDenied,
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    ValidationError(String),
}

pub struct ErrorResponse {
    pub error: AppError,
}

impl IntoResponse for AppError {
    #[allow(unused)]
    fn into_response(self) -> Response {
        let (status) = match self {
            AppError::AlreadyExists => StatusCode::CONFLICT,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized | AppError::InvalidToken | AppError::MissingToken => {
                StatusCode::UNAUTHORIZED
            }
            AppError::Database(_) | AppError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AccessDenied => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) | AppError::InvalidId => StatusCode::BAD_REQUEST,
            AppError::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
        };

        let message = self.to_string();
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23505") => {
                AppError::AlreadyExists
            }
            sqlx::Error::RowNotFound => AppError::NotFound,
            sqlx::Error::Database(db_err) => {
                tracing::error!("DB error code: {:?}", db_err.code());
                tracing::error!("DB error message: {:?}", db_err.message());
                AppError::Database(sqlx::Error::Database(db_err))
            }
            _ => AppError::Database(e),
        }
    }
}
