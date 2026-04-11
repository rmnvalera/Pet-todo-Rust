use axum::{
    Json, async_trait,
    extract::{FromRequest, Request},
};
use errors::AppError;
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, AppError> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|_| AppError::BadRequest("Invalid JSON".into()))?;

        value
            .validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        Ok(ValidatedJson(value))
    }
}
