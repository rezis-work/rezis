use rezis::{json, Controller, JsonResult, RouteBuilder, ValidatedJson};

use super::users_dto::CreateUserDto;
use super::users_service::{User, UsersService};

#[derive(Clone)]
pub struct UsersController {
    service: UsersService,
}

impl UsersController {
    pub fn new(service: UsersService) -> Self {
        Self { service }
    }

    async fn create_user(&self, dto: CreateUserDto) -> JsonResult<User> {
        Ok(json(self.service.create(dto).await))
    }
}

impl Controller for UsersController {
    fn register<'a>(&self, routes: RouteBuilder<'a>) -> RouteBuilder<'a> {
        routes
            .get("/users", {
                let c = self.clone();
                || async move { json(c.service.find_all().await) }
            })
            .post("/users", {
                let c = self.clone();
                |ValidatedJson(body)| async move { c.create_user(body).await }
            })
    }
}
