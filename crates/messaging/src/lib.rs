use std::sync::Arc;

use async_nats::Client;
use async_trait::async_trait;
use futures::StreamExt;
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions,
        QueueBindOptions, QueueDeclareOptions,
    },
    types::FieldTable,
};
use tokio::task::JoinHandle;

use crate::error::MessagingError;

pub mod error;

#[async_trait]
pub trait MessageBus: Send + Sync {
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), MessagingError>;
    async fn subscribe(
        &self,
        topic: &str,
        handler: Arc<dyn MessageHandler>,
    ) -> Result<JoinHandle<()>, MessagingError>;
}

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, payload: Vec<u8>);
}

pub struct NatsMessageBus {
    pub cli: Client,
    pub queue_group: String,
}
impl NatsMessageBus {
    pub async fn new(url: &str, queue_group: &str) -> Result<Self, MessagingError> {
        let nc = async_nats::connect(&url)
            .await
            .map_err(|e| MessagingError::Connection(format!("Nats: {}", e)))?;

        Ok(Self {
            cli: nc,
            queue_group: queue_group.to_string(),
        })
    }
}

#[async_trait]
impl MessageBus for NatsMessageBus {
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), MessagingError> {
        self.cli
            .publish(topic.to_string(), payload.to_vec().into())
            .await
            .map_err(|e| MessagingError::Publish(e.to_string()))?;

        Ok(())
    }

    async fn subscribe(
        &self,
        topic: &str,
        handler: Arc<dyn MessageHandler>,
    ) -> Result<JoinHandle<()>, MessagingError> {
        let mut subscriber = self
            .cli
            .queue_subscribe(topic.to_string(), self.queue_group.clone())
            .await
            .map_err(|e| MessagingError::Subscribe(e.to_string()))?;

        let handle = tokio::spawn(async move {
            while let Some(message) = subscriber.next().await {
                handler.handle(message.payload.to_vec()).await
            }

            tracing::error!("NATS subscription ended unexpectedly!");
            std::process::exit(1);
        });
        Ok(handle)
    }
}

pub struct RabbitMessageBus {
    pub connection: Connection,
    pub channel: Channel,
    pub queue: String,
    pub exchange: String,
}

impl RabbitMessageBus {
    pub async fn new(url: &str, queue: &str, exchange: &str) -> Result<Self, MessagingError> {
        let connection = Connection::connect(url, ConnectionProperties::default())
            .await
            .map_err(|e| MessagingError::Connection(format!("Rabbit: {}", e)))?;

        let channel = connection.create_channel().await.unwrap_or_else(|e| {
            tracing::error!("Failed to create channel to Rabbit: {}", e);
            std::process::exit(1);
        });

        channel
            .queue_declare(queue, QueueDeclareOptions::default(), FieldTable::default())
            .await
            .map_err(|e| MessagingError::Publish(e.to_string()))
            .unwrap_or_else(|e| {
                tracing::error!("Failed to queue declare to Rabbit: {}", e);
                std::process::exit(1);
            });

        channel
            .exchange_declare(
                exchange,
                ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| MessagingError::Publish(e.to_string()))
            .unwrap_or_else(|e| {
                tracing::error!("Failed to queue declare to Rabbit: {}", e);
                std::process::exit(1);
            });

        Ok(RabbitMessageBus {
            connection,
            channel,
            exchange: exchange.to_string(),
            queue: queue.to_string(),
        })
    }
}

#[async_trait]
impl MessageBus for RabbitMessageBus {
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), MessagingError> {
        self.channel
            .basic_publish(
                &self.exchange,
                topic,
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default(),
            )
            .await
            .map_err(|e| MessagingError::Publish(e.to_string()))?
            .await
            .map_err(|e| MessagingError::Publish(e.to_string()))?;

        Ok(())
    }

    async fn subscribe(
        &self,
        topic: &str,
        handler: Arc<dyn MessageHandler>,
    ) -> Result<JoinHandle<()>, MessagingError> {
        self.channel
            .queue_bind(
                &self.queue,
                &self.exchange,
                topic,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| MessagingError::Subscribe(e.to_string()))?;

        let mut consumer = self
            .channel
            .basic_consume(
                &self.queue,
                "",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| MessagingError::Subscribe(e.to_string()))?;

        let handele = tokio::spawn(async move {
            while let Some(delivery) = consumer.next().await {
                match delivery {
                    Ok(delivery) => {
                        handler.handle(delivery.data.clone()).await;
                        let _ = delivery.ack(BasicAckOptions::default()).await;
                    }
                    Err(e) => {
                        tracing::error!("Rabbit consume error: {}", e);
                    }
                }
            }

            tracing::error!("Rabbit subscription ended unexpectedly!");
            std::process::exit(1);
        });
        Ok(handele)
    }
}
