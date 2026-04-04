use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::AppState;

pub mod login;
pub mod me;
pub mod register;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register::handler))
        .route("/login", post(login::handler))
        .route("/me", get(me::handler))
}
