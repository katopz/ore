use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use turso::{Builder, Connection};
use uuid::Uuid;

// Application state with real Turso database
#[derive(Clone)]
struct AppState {
    db: Arc<Connection>,
    db_url: String,
    solana_url: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Record {
    id: String,
    message: String,
    created_at: String,
}

#[tokio::main]
async fn main() {
    // Initialize database URL from environment or default
    let db_url = std::env::var("TURSO_URL").unwrap_or_else(|_| "test.db".to_string());
    println!("🗄️  Initializing database connection to: {}", db_url);

    // Initialize Turso database connection
    let db = match init_database(&db_url).await {
        Ok(db) => {
            println!("✅ Database connection established");
            db
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize Solana URL (using devnet)
    let solana_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    println!("🔗 Solana URL configured: {}", solana_url);

    let app_state = AppState {
        db,
        db_url: db_url.clone(),
        solana_url,
    };

    let app = Router::new()
        .route("/", get(health))
        .route("/status", get(status))
        .route("/db/create", post(create_record))
        .route("/db/list", get(list_records))
        .route("/solana/info", get(solana_info))
        .route("/solana/balance/{pubkey}", get(get_balance))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Phase 3 server (axum + turso + solana) listening on http://0.0.0.0:3000");
    println!("📍 Database: {}", db_url);

    axum::serve(listener, app).await.unwrap();
}

/// Initialize database connection
async fn init_database(db_url: &str) -> Result<Arc<Connection>, Box<dyn std::error::Error>> {
    let db = Builder::new_local(db_url).build().await?;
    let conn = db.connect()?;
    let conn = Arc::new(conn);

    // Create test table if it doesn't exist
    let create_table_sql = r#"
        CREATE TABLE IF NOT EXISTS records (
            id TEXT PRIMARY KEY,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
    "#;

    conn.execute(create_table_sql, ()).await?;
    println!("✅ Database table initialized");

    Ok(conn)
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "phase": "3-axum-turso-solana",
        "service": "phase3-lab",
        "arch": "x86_64",
        "database": "turso",
        "blockchain": "solana-devnet"
    }))
}

async fn status(State(state): State<AppState>) -> Json<Value> {
    // Test database connection
    let db_status = match state.db.execute("SELECT 1", ()).await {
        Ok(_) => "connected".to_string(),
        Err(e) => format!("error: {}", e),
    };

    Json(json!({
        "service": "phase3-axum-turso-solana",
        "status": "running",
        "phase": 3,
        "database": "turso",
        "database_url": state.db_url,
        "db_status": db_status,
        "blockchain": "solana-devnet",
        "ready": true
    }))
}

async fn create_record(State(state): State<AppState>) -> Json<Value> {
    let id = Uuid::new_v4().to_string();
    let message = format!("Test record created at {}", chrono::Utc::now());
    let created_at = chrono::Utc::now().to_rfc3339();

    // Insert into Turso database
    let insert_sql = "INSERT INTO records (id, message, created_at) VALUES (?, ?, ?)";

    match state.db.execute(insert_sql, (id.as_str(), message.as_str(), created_at.as_str())).await {
        Ok(_) => {
            Json(json!({
                "success": true,
                "id": id,
                "message": message,
                "created_at": created_at,
                "database": "turso"
            }))
        }
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Database insert failed: {}", e),
                "id": id
            }))
        }
    }
}

async fn list_records(State(state): State<AppState>) -> Json<Value> {
    // Query all records from Turso database
    let select_sql = "SELECT id, message, created_at FROM records ORDER BY created_at DESC";

    match state.db.query(select_sql, ()).await {
        Ok(mut rows) => {
            let mut records = Vec::new();

            loop {
                match rows.next().await {
                    Ok(Some(row)) => {
                        let record = Record {
                            id: match row.get::<String>(0) {
                                Ok(val) => val,
                                Err(e) => {
                                    return Json(json!({
                                        "success": false,
                                        "error": format!("Failed to get id: {}", e),
                                        "count": 0,
                                        "records": []
                                    }));
                                }
                            },
                            message: match row.get::<String>(1) {
                                Ok(val) => val,
                                Err(e) => {
                                    return Json(json!({
                                        "success": false,
                                        "error": format!("Failed to get message: {}", e),
                                        "count": 0,
                                        "records": []
                                    }));
                                }
                            },
                            created_at: match row.get::<String>(2) {
                                Ok(val) => val,
                                Err(e) => {
                                    return Json(json!({
                                        "success": false,
                                        "error": format!("Failed to get created_at: {}", e),
                                        "count": 0,
                                        "records": []
                                    }));
                                }
                            },
                        };
                        records.push(record);
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Json(json!({
                            "success": false,
                            "error": format!("Row processing failed: {}", e),
                            "count": 0,
                            "records": []
                        }));
                    }
                }
            }

            Json(json!({
                "success": true,
                "count": records.len(),
                "records": records,
                "database": "turso"
            }))
        }
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Database query failed: {}", e),
                "count": 0,
                "records": []
            }))
        }
    }
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
