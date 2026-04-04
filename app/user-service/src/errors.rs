use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
#[allow(unused)]
pub enum AppError {
    Database(sqlx::Error),
    UserAlreadyExists,
    NotFound,
    Unauthorized,
    InternalError,
    MissingToken,
    InvalidToken,
}

// Вот магия — Axum умеет превращать это в HTTP ответ автоматически
impl IntoResponse for AppError {
    #[allow(unused)]
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            AppError::Unauthorized | AppError::InvalidToken | AppError::MissingToken => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials")
            }
            AppError::Database(_) | AppError::InternalError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

// Позволяет использовать ? для sqlx::Error
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23505") => {
                AppError::UserAlreadyExists
            }
            sqlx::Error::Database(db_err) => {
                // Временно — посмотреть что реально приходит
                println!("DB error code: {:?}", db_err.code());
                println!("DB error message: {:?}", db_err.message());
                AppError::Database(sqlx::Error::Database(db_err))
            }
            _ => AppError::Database(e),
        }
    }
}
