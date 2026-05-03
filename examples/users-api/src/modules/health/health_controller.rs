use rezis::{json, Controller, RouteBuilder};

#[derive(Clone, Copy)]
pub struct HealthController;

impl Controller for HealthController {
    fn register<'a>(&self, routes: RouteBuilder<'a>) -> RouteBuilder<'a> {
        routes.get("/health", || async {
            json(serde_json::json!({ "status": "ok" }))
        })
    }
}
