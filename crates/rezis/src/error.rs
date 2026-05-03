use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum RezisError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

impl RezisError {
    pub fn code(&self) -> &'static str {
        match self {
            RezisError::BadRequest(_) => "BAD_REQUEST",
            RezisError::Unauthorized(_) => "UNAUTHORIZED",
            RezisError::Forbidden(_) => "FORBIDDEN",
            RezisError::NotFound(_) => "NOT_FOUND",
            RezisError::Validation(_) => "VALIDATION_ERROR",
            RezisError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            RezisError::BadRequest(_) | RezisError::Validation(_) => StatusCode::BAD_REQUEST,
            RezisError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            RezisError::Forbidden(_) => StatusCode::FORBIDDEN,
            RezisError::NotFound(_) => StatusCode::NOT_FOUND,
            RezisError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        self.to_string()
    }

    /// Attach structured validation details (Phase 3 use); Phase 1 callers can ignore.
    pub fn with_details(self, details: Value) -> DetailedRezisError {
        DetailedRezisError {
            inner: self,
            details: Some(details),
        }
    }
}

/// [`RezisError`] plus optional `details` for `VALIDATION_ERROR`-style payloads.
pub struct DetailedRezisError {
    inner: RezisError,
    details: Option<Value>,
}

impl IntoResponse for DetailedRezisError {
    fn into_response(self) -> Response {
        let status = self.inner.status_code();
        let body = ApiFailure {
            success: false,
            error: ApiErrorBody {
                code: self.inner.code(),
                message: self.inner.message(),
                details: self.details,
            },
        };
        (status, Json(body)).into_response()
    }
}

/// Inner `error` object for failure responses.
#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Failure API envelope: `{ "success": false, "error": { ... } }`.
#[derive(Debug, Serialize)]
pub struct ApiFailure {
    pub success: bool,
    pub error: ApiErrorBody,
}

impl IntoResponse for RezisError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ApiFailure {
            success: false,
            error: ApiErrorBody {
                code: self.code(),
                message: self.message(),
                details: None,
            },
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn not_found_response_shape() {
        let resp = RezisError::NotFound("User not found".into()).into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["success"], false);
        assert_eq!(v["error"]["code"], "NOT_FOUND");
        assert_eq!(v["error"]["message"], "User not found");
    }
}
