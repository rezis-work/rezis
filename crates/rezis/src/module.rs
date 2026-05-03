use axum::Router;
use http::Method;

use crate::controller::{Controller, RouteBuilder};

/// Composable application slice (NestJS-style module).
pub trait Module: Send + Sync + 'static {
    fn register(&self, ctx: &mut ModuleContext<'_>);
}

/// Mutable registration scope shared by nested modules and the root [`crate::RezisApp`](crate::RezisApp).
pub struct ModuleContext<'a> {
    router: &'a mut Router,
    routes: &'a mut Vec<(Method, String)>,
}

impl<'a> ModuleContext<'a> {
    pub(crate) fn new(router: &'a mut Router, routes: &'a mut Vec<(Method, String)>) -> Self {
        Self { router, routes }
    }

    pub fn module(&mut self, child: impl Module) {
        child.register(self);
    }

    pub fn controller<C: Controller>(&mut self, c: C) {
        let rb = RouteBuilder::new(self.router, self.routes);
        let _ = c.register(rb);
    }
}
