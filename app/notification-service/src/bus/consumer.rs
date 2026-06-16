// use crate::grpc::notification_service::notification::Notification;
use crate::state::AppState;

pub async fn run(_state: AppState) {
    // TODO:
    loop {
        // let msg = bus_connection.recv().await;
        // let (user_id, notification) = parse(msg);
        // state.send(user_id, notification);

        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
