use auth_jwt::{Claims, JwtConfig};
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use errors::AppError;
use uuid::Uuid;

#[allow(unused)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync + JwtConfig,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
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
            &jsonwebtoken::DecodingKey::from_secret(state.jwt_secret().as_bytes()),
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
