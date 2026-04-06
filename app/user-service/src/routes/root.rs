use axum::{Json};
use serde_json::{Value, json};

pub async fn root() -> Json<Value> {
    Json(json!({
        "message": format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }))
}