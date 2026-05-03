use axum::handler::Handler;
use axum::Router;
use http::Method;

/// Registers HTTP routes for a module (NestJS-style controller).
pub trait Controller: Send + Sync + 'static {
    fn register<'a>(&self, routes: RouteBuilder<'a>) -> RouteBuilder<'a>;
}

/// Fluent route registration; same handler constraints as [`crate::RezisApp::get`](crate::RezisApp::get).
pub struct RouteBuilder<'a> {
    router: &'a mut Router,
    routes: &'a mut Vec<(Method, String)>,
}

impl<'a> RouteBuilder<'a> {
    pub(crate) fn new(router: &'a mut Router, routes: &'a mut Vec<(Method, String)>) -> Self {
        Self { router, routes }
    }

    pub fn get<H, T>(self, path: impl Into<String>, handler: H) -> Self
    where
        H: Handler<T, ()> + Clone + Send + Sync + 'static,
        T: 'static,
    {
        crate::routing::add_get(self.router, self.routes, path, handler);
        self
    }

    pub fn post<H, T>(self, path: impl Into<String>, handler: H) -> Self
    where
        H: Handler<T, ()> + Clone + Send + Sync + 'static,
        T: 'static,
    {
        crate::routing::add_post(self.router, self.routes, path, handler);
        self
    }
}
