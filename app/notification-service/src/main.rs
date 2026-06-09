use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use axum::{Router, routing::get};
use entities::tasks::Model as Task;
use messaging::{MessageBus, MessageHandler, NatsMessageBus, RabbitMessageBus};
use settings::Settings;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::routes::{health::health, root::root};

pub mod routes;

pub struct TaskCreatedHandler;

#[async_trait]
impl MessageHandler for TaskCreatedHandler {
    async fn handle(&self, payload: Vec<u8>) {
        let task = match serde_json::from_slice::<Task>(&payload) {
            Ok(task) => task,
            Err(e) => {
                tracing::error!("Failed to parse task: {}", e);
                return;
            }
        };
        tracing::info!("get event task created: {:?}", task);
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
    let settings = Settings::new(Option::Some("3003".to_string()), Option::None)
        .context("Configuration Error")?;
    let service_port = &settings.port.clone();

    tracing::info!("{:?}", settings);

    tracing::info!(
        "{} start on port {}..",
        env!("CARGO_PKG_NAME"),
        service_port
    );

    let bus: Arc<dyn MessageBus> = match settings.messaging.provider.as_str() {
        "nats" => {
            Arc::new(NatsMessageBus::new(&settings.messaging.url, env!("CARGO_PKG_NAME")).await?)
        }
        "rabbitmq" => Arc::new(
            RabbitMessageBus::new(&settings.messaging.url, env!("CARGO_PKG_NAME"), "event").await?,
        ),
        _ => panic!("Unknown messaging provider"),
    };

    bus.subscribe("task.created", Arc::new(TaskCreatedHandler))
        .await?;

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
        .layer(on_failure);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", service_port)).await?;
    axum::serve(listener, app).await?;

    anyhow::Ok(())
}
