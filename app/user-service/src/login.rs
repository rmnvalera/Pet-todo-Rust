use std::time::Instant;

use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use tracing::{info, info_span};
use uuid::Uuid;

pub async fn access_log_middleware(request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4();
    let start = Instant::now();

    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();
    let ip = request.headers().get("x-forwarded-for").cloned();


    // передаем дальше
    let response = next.run(request).await;

    let latency = start.elapsed();

    let status = response.status();

    let span = info_span!(
        "http_request",
        request_id = %request_id,
        method = %method,
        uri = %uri,
    );
    let _enter = span.enter();

    
    info!(
        version = ?version,
        status = %status.as_u16(),
        latency_ms = latency.as_millis(),
        ip = ?ip,
        "http_access_log"
    );

    response
}
