//! Shared route registration for [`crate::RezisApp`](crate::RezisApp) and [`crate::RouteBuilder`](crate::RouteBuilder).

use axum::handler::Handler;
use axum::routing::{get, post};
use axum::Router;
use http::Method;

pub(crate) fn add_get<H, T>(
    router: &mut Router,
    routes: &mut Vec<(Method, String)>,
    path: impl Into<String>,
    handler: H,
) where
    H: Handler<T, ()> + Clone + Send + Sync + 'static,
    T: 'static,
{
    let path = path.into();
    routes.push((Method::GET, path.clone()));
    *router = router.clone().route(&path, get(handler));
}

pub(crate) fn add_post<H, T>(
    router: &mut Router,
    routes: &mut Vec<(Method, String)>,
    path: impl Into<String>,
    handler: H,
) where
    H: Handler<T, ()> + Clone + Send + Sync + 'static,
    T: 'static,
{
    let path = path.into();
    routes.push((Method::POST, path.clone()));
    *router = router.clone().route(&path, post(handler));
}

pub(crate) fn add_health_route(
    router: &mut Router,
    routes: &mut Vec<(Method, String)>,
    path: impl Into<String>,
) {
    let path = path.into();
    routes.push((Method::GET, path.clone()));
    *router = router.clone().route(
        &path,
        get(|| async { crate::json(serde_json::json!({ "status": "ok" })) }),
    );
}
