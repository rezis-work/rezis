use rezis::{json, RezisApp};

#[tokio::main]
async fn main() {
    RezisApp::new()
        .get("/", || async { json("Hello from Rezis") })
        .with_health("/health")
        .listen_from_env()
        .await;
}
