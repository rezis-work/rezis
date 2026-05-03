use rezis::{Module, ModuleContext};

use super::users_controller::UsersController;
use super::users_service::UsersService;

pub struct UsersModule;

impl UsersModule {
    pub fn new() -> Self {
        Self
    }
}

impl Module for UsersModule {
    fn register(&self, ctx: &mut ModuleContext<'_>) {
        let service = UsersService::new();
        let controller = UsersController::new(service);
        ctx.controller(controller);
    }
}
