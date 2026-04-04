use std::sync::Arc;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::request::Parts,
};
use uuid::Uuid;

use crate::{AppState, errors::AppError, routes::users::login::Claims};

#[allow(unused)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        let State(state) = State::<Arc<AppState>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InternalError)?;

        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or(AppError::MissingToken)?
            .to_str()
            .map_err(|_| AppError::InvalidToken)?;
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::InvalidToken)?;

        // 3. Декодируй JWT через jsonwebtoken::decode
        let decode = jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(state.settings.auth.jwt_secret.as_bytes()),
            &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256),
        )
        .map_err(|_| AppError::InvalidToken)?;
        // 4. Верни AuthUser { user_id, email }

        let claims = &decode.claims;
        let uuid = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InternalError)?;

        Ok(Self {
            user_id: uuid,
            email: claims.email.clone(),
        })
    }
}
