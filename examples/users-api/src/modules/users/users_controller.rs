use rezis::{json, Controller, RouteBuilder};

use super::users_service::UsersService;

#[derive(Clone)]
pub struct UsersController {
    service: UsersService,
}

impl UsersController {
    pub fn new(service: UsersService) -> Self {
        Self { service }
    }
}

impl Controller for UsersController {
    fn register<'a>(&self, routes: RouteBuilder<'a>) -> RouteBuilder<'a> {
        routes.get("/users", {
            let c = self.clone();
            || async move { json(c.service.find_all().await) }
        })
    }
}
