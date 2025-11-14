use axum::{
    extract::Path,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(health))
        .route("/test/:name", get(test))
        .route("/status", get(status));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Axum-only server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "phase": "1-axum-only",
        "service": "axum-only-lab"
    }))
}

async fn test(Path(name): Path<String>) -> Json<Value> {
    Json(json!({
        "message": format!("Hello, {}!", name),
        "phase": "1-axum-only"
    }))
}

async fn status() -> Json<Value> {
    Json(json!({
        "service": "axum-only",
        "status": "running",
        "features": ["axum", "tokio"],
        "phase": 1
    }))
}
