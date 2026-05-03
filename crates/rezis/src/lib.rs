//! Rezis — NestJS-inspired Rust backend framework for clean, modular APIs.

mod app;
mod error;
mod response;

pub use app::RezisApp;
pub use error::{ApiErrorBody, ApiFailure, DetailedRezisError, RezisError};
pub use response::{json, ApiSuccess};
pub use serde_json;

use axum::Json;

/// Typed JSON handler result using the standard success envelope.
pub type JsonResult<T> = Result<Json<ApiSuccess<T>>, RezisError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn router_root_and_health_return_success_envelope() {
        let app = RezisApp::new()
            .get("/", || async { json("Hello from Rezis") })
            .with_health("/health")
            .into_router();

        let root = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(root.status(), StatusCode::OK);
        let bytes = to_bytes(root.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["success"], true);
        assert_eq!(v["data"], "Hello from Rezis");

        let health = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(health.status(), StatusCode::OK);
        let bytes = to_bytes(health.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["success"], true);
        assert_eq!(v["data"]["status"], "ok");
    }
}
