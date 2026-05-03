use axum::handler::Handler;
use axum::routing::{get, post};
use axum::Router;
use http::Method;
use std::borrow::Cow;

/// NestJS-style fluent app builder over Axum.
pub struct RezisApp {
    router: Router,
    routes: Vec<(Method, String)>,
}

impl RezisApp {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            routes: Vec::new(),
        }
    }

    /// Register a GET handler (same constraints as Axum's [`get`]).
    pub fn get<H, T>(mut self, path: impl Into<String>, handler: H) -> Self
    where
        H: Handler<T, ()> + Clone + Send + Sync + 'static,
        T: 'static,
    {
        let path = path.into();
        self.routes.push((Method::GET, path.clone()));
        self.router = self.router.route(&path, get(handler));
        self
    }

    /// Register a POST handler (same constraints as Axum's [`post`]).
    pub fn post<H, T>(mut self, path: impl Into<String>, handler: H) -> Self
    where
        H: Handler<T, ()> + Clone + Send + Sync + 'static,
        T: 'static,
    {
        let path = path.into();
        self.routes.push((Method::POST, path.clone()));
        self.router = self.router.route(&path, post(handler));
        self
    }

    /// Convenience GET handler returning enveloped `{ "status": "ok" }` data.
    pub fn with_health(mut self, path: impl Into<Cow<'static, str>>) -> Self {
        let path = path.into().into_owned();
        self.routes.push((Method::GET, path.clone()));
        self.router = self.router.route(
            &path,
            get(|| async { crate::json(serde_json::json!({ "status": "ok" })) }),
        );
        self
    }

    pub async fn listen<A: tokio::net::ToSocketAddrs>(self, addr: A) {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind server");
        let bound = listener.local_addr().expect("Failed to read bound address");
        println!("Rezis server running on http://{}", bound);
        for (method, path) in &self.routes {
            println!("{} {}", method.as_str(), path);
        }
        axum::serve(listener, self.router)
            .await
            .expect("Server failed");
    }
}

impl Default for RezisApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl RezisApp {
    pub(crate) fn into_router(self) -> Router {
        self.router
    }
}
