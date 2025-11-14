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
    solana_url: String,
    ore_program_id: String,
}

#[tokio::main]
async fn main() {
    // Initialize in-memory database
    let db = MemoryDB::new(Mutex::new(Vec::new()));

    // Initialize Solana URL (using devnet)
    let solana_url = "https://api.devnet.solana.com".to_string();
    println!("✅ Solana URL configured: {}", solana_url);

    // Initialize ORE program ID (using System Program as valid placeholder)
    let ore_program_id = "11111111111111111111111111111111".to_string();
    println!("✅ ORE program ID configured (using System Program as placeholder): {}", ore_program_id);

    let app_state = AppState { db, solana_url, ore_program_id };

    let app = Router::new()
        .route("/", get(health))
        .route("/status", get(status))
        .route("/db/create", post(create_record))
        .route("/db/list", get(list_records))
        .route("/solana/info", get(solana_info))
        .route("/solana/balance/{pubkey}", get(get_balance))
        .route("/ore/info", get(ore_info))
        .route("/ore/balance/{pubkey}", get(get_ore_balance))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Phase 4 server (axum + memory-db + solana + ore) listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "phase": "4-axum-memory-solana-ore",
        "service": "phase4-lab",
        "arch": "x86_64",
        "database": "memory",
        "blockchain": "solana-devnet",
        "program": "ore"
    }))
}

async fn status() -> Json<Value> {
    Json(json!({
        "service": "phase4-axum-memory-solana-ore",
        "status": "running",
        "phase": 4,
        "database": "memory",
        "blockchain": "solana-devnet",
        "program": "ore",
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
    let client = solana_client::rpc_client::RpcClient::new(&state.solana_url);

    match client.get_latest_blockhash() {
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

async fn get_balance(State(state): State<AppState>, axum::extract::Path(pubkey): axum::extract::Path<String>) -> Json<Value> {
    let client = solana_client::rpc_client::RpcClient::new(&state.solana_url);

    // Parse public key
    match pubkey.parse::<solana_sdk::pubkey::Pubkey>() {
        Ok(pubkey) => {
            match client.get_balance(&pubkey) {
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

async fn ore_info(State(state): State<AppState>) -> Json<Value> {
    let client = solana_client::rpc_client::RpcClient::new(&state.solana_url);

    match state.ore_program_id.parse::<solana_sdk::pubkey::Pubkey>() {
        Ok(program_pubkey) => {
            match client.get_account(&program_pubkey) {
                Ok(account) => Json(json!({
                    "success": true,
                    "program_id": state.ore_program_id,
                    "program_owner": account.owner.to_string(),
                    "program_lamports": account.lamports,
                    "program_executable": account.executable,
                    "program_data": account.data.len(),
                    "note": "Mock ORE info - actual ORE API when dependency conflicts resolved"
                })),
                Err(e) => Json(json!({
                    "success": false,
                    "error": e.to_string(),
                    "program_id": state.ore_program_id,
                    "note": "Expected error on devnet - ORE program may not be deployed"
                }))
            }
        }
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Invalid ORE program ID: {}", e),
            "program_id": state.ore_program_id
        }))
    }
}

async fn get_ore_balance(State(state): State<AppState>, axum::extract::Path(pubkey): axum::extract::Path<String>) -> Json<Value> {
    // Parse user public key
    match pubkey.parse::<solana_sdk::pubkey::Pubkey>() {
        Ok(user_pubkey) => {
            // Mock ORE balance query (replace with actual ORE API when dependency conflicts resolved)
            Json(json!({
                "success": true,
                "pubkey": pubkey,
                "note": "Mock ORE balance - dependency conflicts need resolution",
                "mock_balance": 1000000000,
                "mock_balance_formatted": "1.0 ORE",
                "unit": "ORE",
                "program_id": state.ore_program_id
            }))
        }
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Invalid public key: {}", e),
                "pubkey": pubkey
            }))
        }
    }
}
