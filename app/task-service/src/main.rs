use anyhow::{Context, Ok};
use auth_jwt::JwtConfig;
use axum::{Router, routing::get};
use messaging::{MessageBus, NatsMessageBus, RabbitMessageBus};
use settings::Settings;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    db::Database,
    routes::{health::health, root::root, tasks::router},
};

mod db;
mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub settings: Settings,
    pub bus: Arc<dyn MessageBus>,
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
        Option::Some("3002".to_string()),
        Option::Some("task_service".to_string()),
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
        .context("Failed to connect to database")?;

    db.migrate().await.context("Failed to run migrations")?;

    let bus: Arc<dyn MessageBus> = match settings.messaging.provider.as_str() {
        "nats" => {
            Arc::new(NatsMessageBus::new(&settings.messaging.url, env!("CARGO_PKG_NAME")).await?)
        }
        "rabbitmq" => Arc::new(
            RabbitMessageBus::new(&settings.messaging.url, env!("CARGO_PKG_NAME"), "event").await?,
        ),
        _ => panic!("Unknown messaging provider"),
    };

    let state = Arc::new(AppState { db, settings, bus });

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/tasks", router())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", service_port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
