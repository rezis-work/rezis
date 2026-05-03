use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::{Map, Value};
use validator::ValidationErrors;

#[derive(Debug, thiserror::Error)]
pub enum RezisError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    InvalidJson(String),
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
            RezisError::InvalidJson(_) => "INVALID_JSON",
            RezisError::Unauthorized(_) => "UNAUTHORIZED",
            RezisError::Forbidden(_) => "FORBIDDEN",
            RezisError::NotFound(_) => "NOT_FOUND",
            RezisError::Validation(_) => "VALIDATION_ERROR",
            RezisError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            RezisError::BadRequest(_) | RezisError::InvalidJson(_) | RezisError::Validation(_) => {
                StatusCode::BAD_REQUEST
            }
            RezisError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            RezisError::Forbidden(_) => StatusCode::FORBIDDEN,
            RezisError::NotFound(_) => StatusCode::NOT_FOUND,
            RezisError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        self.to_string()
    }

    /// Attach structured validation details for `VALIDATION_ERROR`-style payloads (crate-internal).
    pub(crate) fn with_details(self, details: Value) -> DetailedRezisError {
        DetailedRezisError {
            inner: self,
            details: Some(details),
        }
    }
}

/// Converts [`validator::ValidationErrors`] into `{ "field": ["message", ...], ... }`.
/// Nested structs use dotted paths (`parent.child`).
pub(crate) fn validation_errors_to_details(errors: &ValidationErrors) -> Value {
    let mut map = Map::new();
    append_validation_errors(errors, "", &mut map);
    Value::Object(map)
}

fn append_validation_errors(errors: &ValidationErrors, prefix: &str, map: &mut Map<String, Value>) {
    for (field, errs) in errors.field_errors() {
        let key = if prefix.is_empty() {
            field.to_string()
        } else {
            format!("{}.{}", prefix, field)
        };
        let messages: Vec<Value> = errs
            .iter()
            .map(|e| {
                Value::String(
                    e.message
                        .as_ref()
                        .map(|m| m.as_ref().to_owned())
                        .unwrap_or_else(|| e.code.to_string()),
                )
            })
            .collect();
        map.insert(key, Value::Array(messages));
    }

    for (field, kind) in errors.errors() {
        if let validator::ValidationErrorsKind::Struct(ref nested) = kind {
            let next = if prefix.is_empty() {
                field.to_string()
            } else {
                format!("{}.{}", prefix, field)
            };
            append_validation_errors(nested, &next, map);
        }
    }
}

/// [`RezisError`] plus optional `details` for `VALIDATION_ERROR`-style payloads.
pub(crate) struct DetailedRezisError {
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
    use serde::Deserialize;
    use validator::Validate;

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

    #[derive(Debug, Deserialize, Validate)]
    struct EmailDto {
        #[validate(email)]
        email: String,
    }

    #[test]
    fn validation_errors_to_details_email() {
        let dto = EmailDto {
            email: "not-an-email".into(),
        };
        let err = dto.validate().expect_err("invalid email");
        let details = validation_errors_to_details(&err);
        let emails = details["email"].as_array().expect("email details");
        assert!(!emails.is_empty());
        assert!(emails[0].as_str().unwrap().to_lowercase().contains("email"));
    }
}
