use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::{DbErr, RuntimeErr};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
#[allow(unused)]
pub enum AppError {
    #[error("{0}")]
    Database(String),
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

impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        tracing::error!(error = ?err, "Database error");

        match err {
            DbErr::RecordNotFound(_) => AppError::NotFound,

            DbErr::Exec(RuntimeErr::SqlxError(e)) | DbErr::Query(RuntimeErr::SqlxError(e)) => {
                if let Some(pg) = e.as_database_error() {
                    match pg.code().as_deref() {
                        Some("23505") => {
                            return AppError::AlreadyExists;
                        }
                        Some("23503") => {
                            return AppError::AlreadyExists;
                        }

                        _ => {
                            tracing::error!(
                                code = ?pg.code(),
                                message = ?pg.message(),
                                "Unhandled postgres error"
                            );
                        }
                    }
                }

                AppError::InternalError
            }

            _ => AppError::InternalError,
        }
    }
}
