mod app_module;
mod modules;

use app_module::AppModule;
use rezis::RezisApp;

#[tokio::main]
async fn main() {
    RezisApp::new()
        .module(AppModule::new())
        .listen_from_env()
        .await;
}
