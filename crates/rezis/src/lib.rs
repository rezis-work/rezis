//! Rezis — NestJS-inspired Rust backend framework for clean, modular APIs.

pub mod app;
pub mod config;
pub mod controller;
pub mod error;
mod logging;
pub mod module;
pub mod response;
mod routing;
pub mod validation;

pub use app::RezisApp;
pub use config::RezisConfig;
pub use controller::{Controller, RouteBuilder};
pub use error::{ApiErrorBody, ApiFailure, RezisError};
pub use module::{Module, ModuleContext};
pub use response::{json, ApiSuccess, JsonResult};
pub use validation::ValidatedJson;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::routing::post;
    use axum::Router;
    use http::{Request, StatusCode};
    use serde::Deserialize;
    use tower::ServiceExt;
    use validator::Validate;

    #[derive(Debug, Deserialize, Validate)]
    struct EmailDto {
        #[validate(email)]
        email: String,
    }

    async fn echo_email(
        ValidatedJson(_payload): ValidatedJson<EmailDto>,
    ) -> JsonResult<&'static str> {
        Ok(json("ok"))
    }

    #[tokio::test]
    async fn validated_json_invalid_json_returns_invalid_json_code() {
        let app = Router::new().route("/t", post(echo_email));

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/t")
                    .header("content-type", "application/json")
                    .body(Body::from("{not-json"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["success"], false);
        assert_eq!(v["error"]["code"], "INVALID_JSON");
    }

    #[tokio::test]
    async fn validated_json_bad_email_returns_validation_envelope() {
        let app = Router::new().route("/t", post(echo_email));

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/t")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"email":"not-valid"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["success"], false);
        assert_eq!(v["error"]["code"], "VALIDATION_ERROR");
        assert_eq!(v["error"]["message"], "Invalid request body");
        let emails = v["error"]["details"]["email"]
            .as_array()
            .expect("email details");
        assert!(!emails.is_empty());
    }

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

    struct RootModule;

    impl Module for RootModule {
        fn register(&self, ctx: &mut ModuleContext<'_>) {
            ctx.module(UsersLeafModule);
            ctx.module(ParallelLeafModule);
        }
    }

    struct UsersLeafModule;

    impl Module for UsersLeafModule {
        fn register(&self, ctx: &mut ModuleContext<'_>) {
            ctx.controller(UsersTestController);
        }
    }

    struct ParallelLeafModule;

    impl Module for ParallelLeafModule {
        fn register(&self, ctx: &mut ModuleContext<'_>) {
            ctx.controller(ExtraTestController);
        }
    }

    #[derive(Clone, Copy)]
    struct UsersTestController;

    impl Controller for UsersTestController {
        fn register<'a>(&self, routes: RouteBuilder<'a>) -> RouteBuilder<'a> {
            routes.get("/users", || async {
                json(vec![
                    serde_json::json!({"id": 1, "name": "Ada"}),
                    serde_json::json!({"id": 2, "name": "Grace"}),
                ])
            })
        }
    }

    #[derive(Clone, Copy)]
    struct ExtraTestController;

    impl Controller for ExtraTestController {
        fn register<'a>(&self, routes: RouteBuilder<'a>) -> RouteBuilder<'a> {
            routes.get("/extra", || async { json("ok") })
        }
    }

    #[tokio::test]
    async fn nested_modules_merge_users_and_parallel_routes() {
        let app = RezisApp::new().module(RootModule).into_router();

        let users = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(users.status(), StatusCode::OK);
        let bytes = to_bytes(users.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["success"], true);
        assert!(v["data"].is_array());
        assert_eq!(v["data"].as_array().unwrap().len(), 2);

        let extra = app
            .oneshot(
                Request::builder()
                    .uri("/extra")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(extra.status(), StatusCode::OK);
        let bytes = to_bytes(extra.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["success"], true);
        assert_eq!(v["data"], "ok");
    }

    #[tokio::test]
    async fn layered_logging_and_cors_still_serve_requests() {
        let app = RezisApp::new()
            .with_logging()
            .with_cors()
            .get("/ping", || async { json("pong") })
            .into_router();

        let resp = app
            .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["success"], true);
        assert_eq!(v["data"], "pong");
    }
}
