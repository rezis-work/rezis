use rezis::{json, RezisApp};

#[tokio::main]
async fn main() {
    RezisApp::new()
        .get("/", || async { json("Hello from Rezis") })
        .with_health("/health")
        .listen("0.0.0.0:3000")
        .await;
}
