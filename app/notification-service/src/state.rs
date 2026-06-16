use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::grpc::notification_service::Notification;

const CHANNEL_CAPACITY: usize = 32;

#[derive(Clone)]
pub struct AppState {
    subscribers: Arc<DashMap<String, broadcast::Sender<Notification>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(DashMap::new()),
        }
    }

    pub fn subscribe(&self, user_id: String) -> broadcast::Receiver<Notification> {
        let sender = self
            .subscribers
            .entry(user_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .clone();

        sender.subscribe()
    }

    pub fn send(&self, user_id: String, notification: Notification) -> bool {
        match self.subscribers.get(&user_id) {
            Some(sender) => sender.send(notification).is_ok(),
            None => false,
        }
    }
}
