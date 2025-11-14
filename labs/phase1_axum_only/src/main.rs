use axum::{
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(health))
        .route("/status", get(status));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Phase 1 server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "phase": "1-axum-only",
        "service": "phase1-lab",
        "arch": "x86_64"
    }))
}

async fn status() -> Json<Value> {
    Json(json!({
        "service": "phase1-axum-only",
        "status": "running",
        "phase": 1,
        "ready": true
    }))
}
