use gloo_net::http::Request;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub owner_id: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct PaginatedTasks {
    pub data: Vec<Task>,
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

pub fn get_token() -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item("token")
        .ok()?
}

pub async fn fetch_tasks(page: i64) -> Result<PaginatedTasks, String> {
    let token = get_token().ok_or("Not Authorization")?;

    let response = Request::get(&format!(
        "http://localhost:3002/tasks?page={}&page_size=10",
        page
    ))
    .header("Authorization", &format!("Bearer {}", token))
    .send()
    .await
    .map_err(|e| e.to_string())?;

    if response.ok() {
        response
            .json::<PaginatedTasks>()
            .await
            .map_err(|e| e.to_string())
    } else if response.status() == 401 {
        Err("Seeion is dead".to_string())
    } else {
        Err(format!("Error: {}", response.status()))
    }
}
