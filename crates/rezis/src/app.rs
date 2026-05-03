use axum::handler::Handler;
use axum::Router;
use http::Method;
use std::borrow::Cow;

use crate::module::{Module, ModuleContext};
use crate::routing::{add_get, add_health_route, add_post};

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

    /// Register a NestJS-style module tree on this app.
    pub fn module(mut self, module: impl Module) -> Self {
        let mut ctx = ModuleContext::new(&mut self.router, &mut self.routes);
        module.register(&mut ctx);
        self
    }

    /// Register a GET handler (same constraints as Axum's [`axum::routing::get`]).
    pub fn get<H, T>(mut self, path: impl Into<String>, handler: H) -> Self
    where
        H: Handler<T, ()> + Clone + Send + Sync + 'static,
        T: 'static,
    {
        add_get(&mut self.router, &mut self.routes, path, handler);
        self
    }

    /// Register a POST handler (same constraints as Axum's [`axum::routing::post`]).
    pub fn post<H, T>(mut self, path: impl Into<String>, handler: H) -> Self
    where
        H: Handler<T, ()> + Clone + Send + Sync + 'static,
        T: 'static,
    {
        add_post(&mut self.router, &mut self.routes, path, handler);
        self
    }

    /// Convenience GET handler returning enveloped `{ "status": "ok" }` data.
    pub fn with_health(mut self, path: impl Into<Cow<'static, str>>) -> Self {
        add_health_route(&mut self.router, &mut self.routes, path.into().into_owned());
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
