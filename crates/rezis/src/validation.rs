use std::ops::Deref;

use axum::body::Body;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Json};
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::{validation_errors_to_details, RezisError};

/// Axum extractor that deserializes JSON and validates with [`validator::Validate`].
///
/// On failure, responds with the standard error envelope (including `INVALID_JSON` or
/// `VALIDATION_ERROR` with field details).
#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<T> Deref for ValidatedJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + Send + Sync + 'static,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e: JsonRejection| RezisError::InvalidJson(e.to_string()).into_response())?;

        value.validate().map_err(|e| {
            RezisError::Validation("Invalid request body".into())
                .with_details(validation_errors_to_details(&e))
                .into_response()
        })?;

        Ok(ValidatedJson(value))
    }
}
