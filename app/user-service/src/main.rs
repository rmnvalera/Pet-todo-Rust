use anyhow::{Context, Ok};
use auth_jwt::JwtConfig;
use axum::{Router, routing::get};
use settings::Settings;
use std::sync::Arc;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = Settings::new(
        Option::Some("3001".to_string()),
        Option::Some("user_service".to_string()),
    )?;
    let service_port = &settings.port.clone();

    tracing::info!("{:?}", settings);

    tracing::info!(
        "{} start on port {}..",
        env!("CARGO_PKG_NAME"),
        service_port
    );

    let db = Database::connect(&settings.db)
        .await
        .context("Database connection error")?;
    db.migrate().await.context("Migration Error")?;

    let state = Arc::new(AppState { db, settings });

    let on_failure = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            tracing::span!(
                Level::INFO,
                "request",
                method = %request.method(),
                uri = %request.uri(),
            )
        })
        .on_response(
            |response: &axum::http::Response<_>,
             latency: std::time::Duration,
             _span: &tracing::Span| {
                tracing::info!(status = %response.status(), latency = ?latency);
            },
        )
        .on_failure(
            |error: ServerErrorsFailureClass,
             latency: std::time::Duration,
             _span: &tracing::Span| {
                tracing::info!(_error = %error, latency = ?latency);
            },
        );
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/users", routes::users::router())
        .layer(on_failure)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", service_port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
