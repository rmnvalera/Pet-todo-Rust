use anyhow::Context;
use async_trait::async_trait;
use entities::tasks::Model as Task;
use messaging::{MessageBus, MessageHandler, NatsMessageBus, RabbitMessageBus};
use settings::Settings;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    grpc::notification_service::{
        NotificationServiceImpl,
        notification::notification_service_server::NotificationServiceServer,
    },
    state::AppState,
};

pub struct TaskCreatedHandler;

pub mod bus;
pub mod grpc;
pub mod state;

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
    let settings = Settings::new(Option::Some("50051".to_string()), Option::None)
        .context("Configuration Error")?;
    let service_port = format!("0.0.0.0:{}", &settings.port.clone()).parse()?;

    let app_state = AppState::new();

    tracing::info!("{:?}", settings);

    tracing::info!(
        "{} start on port {}..",
        env!("CARGO_PKG_NAME"),
        service_port
    );

    let bus_state = app_state.clone();
    tokio::spawn(async move {
        bus::consumer::run(bus_state).await;
    });

    let bus: Arc<dyn MessageBus> = match settings.messaging.provider.as_str() {
        "nats" => {
            Arc::new(NatsMessageBus::new(&settings.messaging.url, env!("CARGO_PKG_NAME")).await?)
        }
        "rabbitmq" => Arc::new(
            RabbitMessageBus::new(&settings.messaging.url, env!("CARGO_PKG_NAME"), "event").await?,
        ),
        _ => panic!("Unknown messaging provider"),
    };

    let _handle = bus
        .subscribe("task.created", Arc::new(TaskCreatedHandler))
        .await?;

    // handle.await?;

    let service = NotificationServiceImpl::new(app_state);

    tracing::info!("gRPC server listener on {service_port}");

    tonic::transport::Server::builder()
        .add_service(NotificationServiceServer::new(service))
        .serve(service_port)
        .await?;

    anyhow::Ok(())
}
