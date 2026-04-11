use entities::tasks::{Priority, Task, TaskStatus};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTaskRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Title must be between 1 and 255 characters"
    ))]
    pub title: String,
    #[validate(length(max = 1000, message = "Description too long"))]
    pub description: Option<String>,
    pub priority: Option<Priority>,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub owner_id: String,
    pub created_at: String,
}

impl From<Task> for TaskResponse {
    fn from(task: Task) -> Self {
        Self {
            id: task.id.to_string(),
            title: task.title,
            description: task.description,
            status: format!("{}", task.status),
            priority: format!("{}", task.priority),
            owner_id: task.owner_id.to_string(),
            created_at: task.created_at.to_string(),
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct UpdateTaskRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub search: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl TaskFilter {
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1)
    }

    pub fn per_page(&self) -> u32 {
        self.per_page.unwrap_or(20).min(100)
    }

    pub fn offset(&self) -> u32 {
        (self.page() - 1) * self.per_page()
    }
}
