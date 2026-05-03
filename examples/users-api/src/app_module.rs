use rezis::{Module, ModuleContext};

use crate::modules::health::health_module::HealthModule;
use crate::modules::users::users_module::UsersModule;

pub struct AppModule;

impl AppModule {
    pub fn new() -> Self {
        Self
    }
}

impl Module for AppModule {
    fn register(&self, ctx: &mut ModuleContext<'_>) {
        ctx.module(HealthModule::new());
        ctx.module(UsersModule::new());
    }
}
