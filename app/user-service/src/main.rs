use auth_jwt::JwtConfig;
use axum::{Router, routing::get};
use settings::Settings;
use std::sync::Arc;

use crate::{
    db::Database,
    routes::{health::health, root::root},
};

mod db;
mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub settings: Settings,
}

impl JwtConfig for AppState {
    fn jwt_secret(&self) -> &str {
        &self.settings.jwt.secret
    }
}

#[tokio::main]
async fn main() {
    let settings = Settings::new(
        Option::Some("3001".to_string()),
        Option::Some("user_service".to_string()),
    )
    .unwrap();
    let service_port = &settings.port.clone();

    println!("{:?}", settings);

    println!(
        "{} start on port {}..",
        env!("CARGO_PKG_NAME"),
        service_port
    );

    let db = Database::connect(&settings.db).await.unwrap();
    db.migrate()
        .await
        .map_err(|e| format!("Migration Error: {}", e.to_string()))
        .unwrap();

    let state = Arc::new(AppState { db, settings });

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/users", routes::users::router())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", service_port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
