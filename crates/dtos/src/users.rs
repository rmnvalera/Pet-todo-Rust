use entities::users::User;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JwtResponse {
    pub token: String,
    pub user: UserResponse,
}

impl From<(String, User)> for JwtResponse {
    fn from(value: (String, User)) -> Self {
        Self {
            token: value.0,
            user: value.1.into(),
        }
    }
}
