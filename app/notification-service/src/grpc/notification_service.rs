use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Request, Response, Status};

use crate::state::AppState;

pub mod notification {
    tonic::include_proto!("pet_todo.notification.v1");
}

pub use notification::{
    Notification, SendNotificationRequest, SendNotificationResponse, SubscribeRequest,
    notification_service_server::NotificationService,
};

pub struct NotificationServiceImpl {
    state: AppState,
}

impl NotificationServiceImpl {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl NotificationService for NotificationServiceImpl {
    async fn send_notification(
        &self,
        request: Request<SendNotificationRequest>,
    ) -> Result<Response<SendNotificationResponse>, Status> {
        let req = request.into_inner();

        let notification = Notification {
            topic: req.topic,
            message: req.message,
            created_at: chrono::Utc::now().timestamp(),
        };

        let delivered = self.state.send(req.user_id, notification);

        Ok(Response::new(SendNotificationResponse { delivered }))
    }

    type SubscribeNotificationsStream =
        std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<Notification, Status>> + Send>>;

    async fn subscribe_notifications(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeNotificationsStream>, Status> {
        let user_id = request.into_inner().user_id;
        let receiver = self.state.subscribe(user_id);

        let stream = BroadcastStream::new(receiver)
            .map(|result| result.map_err(|e| Status::internal(format!("stream error: {e}"))));

        Ok(Response::new(Box::pin(stream)))
    }
}
