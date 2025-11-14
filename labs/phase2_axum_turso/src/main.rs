use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// In-memory database
type MemoryDB = Arc<Mutex<Vec<Record>>>;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Record {
    id: String,
    message: String,
    created_at: String,
}

// Application state - needs Clone for Axum
#[derive(Clone)]
struct AppState {
    db: MemoryDB,
}

#[tokio::main]
async fn main() {
    // Initialize in-memory database
    let db = MemoryDB::new(Mutex::new(Vec::new()));
    let app_state = AppState { db };
    
    let app = Router::new()
        .route("/", get(health))
        .route("/status", get(status))
        .route("/db/create", post(create_record))
        .route("/db/list", get(list_records))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Phase 2 server (axum + memory-db) listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "phase": "2-axum-memory",
        "service": "phase2-lab",
        "arch": "x86_64",
        "database": "memory"
    }))
}

async fn status() -> Json<Value> {
    Json(json!({
        "service": "phase2-axum-memory",
        "status": "running",
        "phase": 2,
        "database": "memory",
        "ready": true
    }))
}

async fn create_record(State(state): State<AppState>) -> Json<Value> {
    let id = Uuid::new_v4().to_string();
    let message = format!("Test record created at {}", chrono::Utc::now());
    let created_at = chrono::Utc::now().to_rfc3339();
    
    let record = Record {
        id: id.clone(),
        message: message.clone(),
        created_at: created_at.clone(),
    };
    
    // Store in memory
    let mut db = state.db.lock().unwrap();
    db.push(record);
    
    Json(json!({
        "success": true,
        "id": id,
        "message": message,
        "created_at": created_at
    }))
}

async fn list_records(State(state): State<AppState>) -> Json<Value> {
    let db = state.db.lock().unwrap();
    let count = db.len();
    
    Json(json!({
        "success": true,
        "count": count,
        "records": &*db
    }))
}
