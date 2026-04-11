use std::sync::Arc;

use async_trait::async_trait;
use axum::{Router, routing::get};
use entities::tasks::Task;
use messaging::{MessageBus, MessageHandler, NatsMessageBus, RabbitMessageBus};
use settings::Settings;

use crate::routes::{health::health, root::root};

pub mod routes;

pub struct TaskCreatedHandler;

#[async_trait]
impl MessageHandler for TaskCreatedHandler {
    async fn handle(&self, payload: Vec<u8>) {
        // 1. Десериализуй payload в структуру
        let task = match serde_json::from_slice::<Task>(&payload) {
            Ok(task) => task,
            Err(e) => {
                eprintln!("Failed to parse task: {}", e);
                return;
            }
        };
        // 2. Залогируй или "отправь уведомление"
        println!("get event task created: {:?}", task);
    }
}

#[tokio::main]
async fn main() {
    let settings = Settings::new(Option::Some("3003".to_string()), Option::None).unwrap();
    let service_port = &settings.port.clone();

    println!("{:?}", settings);

    println!(
        "{} start on port {}..",
        env!("CARGO_PKG_NAME"),
        service_port
    );

    let bus: Arc<dyn MessageBus> = match settings.messaging.provider.as_str() {
        "nats" => {
            Arc::new(NatsMessageBus::new(&settings.messaging.url, env!("CARGO_PKG_NAME")).await)
        }
        "rabbitmq" => Arc::new(
            RabbitMessageBus::new(&settings.messaging.url, env!("CARGO_PKG_NAME"), "event").await,
        ),
        _ => panic!("Unknown messaging provider"),
    };

    bus.subscribe("task.created", Arc::new(TaskCreatedHandler))
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed subscribe task.created: {}", e);
            std::process::exit(1);
        });

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", service_port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
