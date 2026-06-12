use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use entities::tasks::Model as Task;
use messaging::{MessageBus, MessageHandler, NatsMessageBus, RabbitMessageBus};
use settings::Settings;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    let handle = bus
        .subscribe("task.created", Arc::new(TaskCreatedHandler))
        .await?;

    handle.await?;
    anyhow::Ok(())
}
