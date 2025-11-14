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
type MemoryDB = Arc<Mutex<Vec<Record>>;

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
    solana_client: Option<solana_client::rpc_client::RpcClient>,
}

#[tokio::main]
async fn main() {
    // Initialize in-memory database
    let db = MemoryDB::new(Mutex::new(Vec::new()));
    
    // Initialize Solana client (using devnet)
    let solana_client = match solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com") {
        client => {
            println!("✅ Solana client initialized");
            Some(client)
        }
        Err(e) => {
            println!("❌ Failed to initialize Solana client: {}", e);
            None
        }
    };
    
    let app_state = AppState { db, solana_client };
    
    let app = Router::new()
        .route("/", get(health))
        .route("/status", get(status))
        .route("/db/create", post(create_record))
        .route("/db/list", get(list_records))
        .route("/solana/info", get(solana_info))
        .route("/solana/balance/:pubkey", get(get_balance))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Phase 3 server (axum + memory-db + solana) listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "phase": "3-axum-memory-solana",
        "service": "phase3-lab",
        "arch": "x86_64",
        "database": "memory",
        "blockchain": "solana-devnet"
    }))
}

async fn status() -> Json<Value> {
    Json(json!({
        "service": "phase3-axum-memory-solana",
        "status": "running",
        "phase": 3,
        "database": "memory",
        "blockchain": "solana-devnet",
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

async fn solana_info(State(state): State<AppState>) -> Json<Value> {
    match &state.solana_client {
        Some(client) => {
            match client.get_latest_blockhash().await {
                Ok(blockhash) => Json(json!({
                    "success": true,
                    "blockchain": "solana-devnet",
                    "latest_blockhash": blockhash.to_string(),
                    "client_status": "connected"
                })),
                Err(e) => Json(json!({
                    "success": false,
                    "error": e.to_string(),
                    "client_status": "error"
                }))
            }
        }
        None => Json(json!({
            "success": false,
            "error": "Solana client not initialized",
            "client_status": "not_available"
        }))
    }
}

async fn get_balance(State(state): State<AppState>, axum::extract::Path(pubkey): axum::extract::Path<String>) -> Json<Value> {
    match &state.solana_client {
        Some(client) => {
            // Parse public key
            match pubkey.parse::<solana_sdk::pubkey::Pubkey>() {
                Ok(pubkey) => {
                    match client.get_balance(&pubkey).await {
                        Ok(balance) => Json(json!({
                            "success": true,
                            "pubkey": pubkey.to_string(),
                            "balance": balance,
                            "balance_sol": balance as f64 / 1_000_000_000.0,
                            "unit": "lamports"
                        })),
                        Err(e) => Json(json!({
                            "success": false,
                            "error": e.to_string(),
                            "pubkey": pubkey.to_string()
                        }))
                    }
                }
                Err(e) => Json(json!({
                    "success": false,
                    "error": format!("Invalid public key: {}", e),
                    "pubkey": pubkey
                }))
            }
        }
        None => Json(json!({
            "success": false,
            "error": "Solana client not initialized"
        }))
    }
}
