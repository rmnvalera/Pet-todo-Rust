use std::sync::Arc;

use axum::{
    Json, Router,
    routing::get,
};
use serde_json::{Value, json};

use crate::{
    db::Database,
    routes::users::{self},
    settings::Settings,
};

mod db;
mod dto;
mod errors;
mod extractors;
mod routes;
mod settings;
mod utils;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub settings: Settings,
}

#[tokio::main]
async fn main() {
    let settings = Settings::new().unwrap();
    let service_port = &settings.port.clone();
    println!(
        "{} start on port {}..",
        env!("CARGO_PKG_NAME"),
        service_port
    );

    let state = Arc::new(AppState {
        db: Database::new(&settings.db_url).await.unwrap(),
        settings,
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/users", users::router())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", service_port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Json<Value> {
    Json(json!({
        "message": format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }))
}
async fn health() -> Json<Value> {
    Json(json!({"status": "ok", "service": "user-service"}))
}
