use anyhow::Ok;
use auth_jwt::JwtConfig;
use axum::{Router, routing::get};
use messaging::{MessageBus, NatsMessageBus, RabbitMessageBus};
use sea_orm::{Database, DatabaseConnection};
use settings::Settings;
use std::sync::Arc;
use task_migration::{Migrator, MigratorTrait};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::routes::{health::health, root::root, tasks::router};

mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
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

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

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

    let conn = Database::connect(settings.db.get_url())
        .await
        .expect("Database connection failed");
    Migrator::up(&conn, None).await.unwrap();

    let bus: Arc<dyn MessageBus> = match settings.messaging.provider.as_str() {
        "nats" => {
            Arc::new(NatsMessageBus::new(&settings.messaging.url, env!("CARGO_PKG_NAME")).await?)
        }
        "rabbitmq" => Arc::new(
            RabbitMessageBus::new(&settings.messaging.url, env!("CARGO_PKG_NAME"), "event").await?,
        ),
        _ => panic!("Unknown messaging provider"),
    };

    let state = Arc::new(AppState {
        db: conn,
        settings,
        bus,
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/tasks", router())
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", service_port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
