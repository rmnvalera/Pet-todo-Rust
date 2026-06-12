use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login(email: String, password: String) -> Result<LoginResponse, String> {
    let body = LoginRequest { email, password };

    let response = Request::post("http://localhost:3001/users/login")
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.ok() {
        response
            .json::<LoginResponse>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(format!("Error: {}", response.status()))
    }
}
