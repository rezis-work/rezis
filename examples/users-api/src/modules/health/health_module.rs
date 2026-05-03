use rezis::{Module, ModuleContext};

use super::health_controller::HealthController;

pub struct HealthModule;

impl HealthModule {
    pub fn new() -> Self {
        Self
    }
}

impl Module for HealthModule {
    fn register(&self, ctx: &mut ModuleContext<'_>) {
        ctx.controller(HealthController);
    }
}
