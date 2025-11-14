use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// ORE API imports
use bincode;
use itertools::Itertools;
use ore_api::prelude::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use spl_associated_token_account;


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

    // Initialize Solana URL (using mainnet for ORE)
    let solana_url = "https://api.mainnet-beta.solana.com".to_string();
    println!("✅ Solana URL configured: {}", solana_url);

    // Initialize ORE program ID
    let ore_program_id = ore_api::id().to_string();
    println!("✅ ORE program ID configured: {}", ore_program_id);

    let app_state = AppState { db, solana_url, ore_program_id };

    let app = Router::new()
        .route("/", get(health))
        .route("/status", get(status))
        .route("/db/create", post(create_record))
        .route("/db/list", get(list_records))
        .route("/solana/info", get(solana_info))
        .route("/solana/balance/{pubkey}", get(get_balance))
        .route("/ore/rounds", get(get_rounds))
        .route("/ore/winner", get(get_round_winner))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3005").await.unwrap();
    println!("🚀 Phase 4 server (axum + memory-db + solana + ore) listening on http://0.0.0.0:3005");

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

async fn get_rounds(State(state): State<AppState>) -> Json<Value> {
    // Create RPC client on demand
    let rpc = RpcClient::new_with_commitment(
        state.solana_url.clone(),
        CommitmentConfig::confirmed(),
    );

    // Get current board info
    let board_pda = board_pda();
    match rpc.get_account(&board_pda.0) {
        Ok(board_account) => {
            match bincode::deserialize::<Board>(&board_account.data) {
                Ok(board) => {
                    // Get current slot to check if round is active
                    match rpc.get_account_data(&solana_sdk::sysvar::clock::ID) {
                        Ok(clock_data) => {
                            match bincode::deserialize::<solana_sdk::clock::Clock>(&clock_data) {
                                Ok(clock) => {
                                    Json(json!({
                                        "success": true,
                                        "program_id": state.ore_program_id,
                                        "board_pda": board_pda.0.to_string(),
                                        "current_round_id": board.round_id,
                                        "start_slot": board.start_slot,
                                        "end_slot": board.end_slot,
                                        "current_slot": clock.slot,
                                        "round_active": clock.slot >= board.start_slot && clock.slot <= board.end_slot,
                                        "program_account_size": board_account.data.len()
                                    }))
                                }
                                Err(e) => Json(json!({
                                    "success": false,
                                    "error": format!("Failed to deserialize clock: {}", e),
                                    "program_id": state.ore_program_id
                                }))
                            }
                        }
                        Err(e) => Json(json!({
                            "success": false,
                            "error": format!("Failed to get clock data: {}", e),
                            "program_id": state.ore_program_id
                        }))
                    }
                }
                Err(e) => Json(json!({
                    "success": false,
                    "error": format!("Failed to deserialize board: {}", e),
                    "program_id": state.ore_program_id
                }))
            }
        }
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to get board account: {}", e),
            "program_id": state.ore_program_id
        }))
    }
}

async fn get_round_winner(State(state): State<AppState>) -> Json<Value> {
    // Create RPC client on demand
    let rpc = RpcClient::new_with_commitment(
        state.solana_url.clone(),
        CommitmentConfig::confirmed(),
    );

    // Get all program accounts
    let program_id = ore_api::id();
    match rpc.get_program_accounts(&program_id) {
        Ok(accounts) => {
            // Look specifically for round accounts
            let mut round_accounts = Vec::new();
            for (pubkey, account) in &accounts {
                if account.data.len() == std::mem::size_of::<Round>() {
                    if let Ok(round) = bincode::deserialize::<Round>(&account.data) {
                        round_accounts.push((pubkey, round));
                    }
                }
            }

            if round_accounts.is_empty() {
                return Json(json!({
                    "success": false,
                    "error": "No round accounts found",
                    "message": "They appear to be cleaned up after expiration. To find winner information, you need to check rounds BEFORE they expire.",
                    "total_accounts": accounts.len()
                }));
            }

            // Sort by round ID
            round_accounts.sort_by_key(|(_, round)| round.id);

            let mut results = Vec::new();
            for (pubkey, round) in &round_accounts {
                let mut round_info = json!({
                    "round_id": round.id,
                    "pubkey": pubkey.to_string(),
                    "top_miner": round.top_miner.to_string(),
                    "top_miner_reward": round.top_miner_reward,
                    "total_deployed": round.total_deployed,
                    "total_winnings": round.total_winnings,
                    "expires_at": round.expires_at
                });

                // Get the random number and winning square if finalized
                if let Some(rng) = round.rng() {
                    let winning_square_index = round.winning_square(rng);
                    let is_split_reward = round.is_split_reward(rng);
                    let did_hit_motherlode = round.did_hit_motherlode(rng);

                    round_info["winning_square_index"] = json!(winning_square_index);
                    round_info["winning_square_row"] = json!(winning_square_index / 5 + 1);
                    round_info["winning_square_col"] = json!(winning_square_index % 5 + 1);
                    round_info["split_reward"] = json!(is_split_reward);
                    round_info["motherlode_hit"] = json!(did_hit_motherlode);
                } else {
                    round_info["status"] = json!("not_finalized");
                    round_info["note"] = json!("No slot hash yet");
                }

                results.push(round_info);
            }

            Json(json!({
                "success": true,
                "round_count": round_accounts.len(),
                "total_program_accounts": accounts.len(),
                "rounds": results
            }))
        }
        Err(e) => Json(json!({
            "success": false,
            "error": format!("Failed to get program accounts: {}", e),
            "program_id": state.ore_program_id
        }))
    }
}
