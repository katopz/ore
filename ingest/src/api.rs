use crate::blockchain::BlockchainClient;
use crate::database::Database;
use crate::ingest::IngestOrchestrator;
use crate::types::{ErrorResponse, RoundListResponse};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Deserialize)]
pub struct ListQueryParams {
    pub limit: Option<i64>,
}

#[derive(Clone)]
pub struct AppState {
    pub db_url: String,
}

/// Initialize API routes
pub fn create_api_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health_check))
        .route("/ingest", get(trigger_ingest))
        .route("/list", get(list_rounds))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}

/// Health check endpoint
async fn health_check() -> Result<Json<HashMap<String, String>>, StatusCode> {
    let mut response = HashMap::new();
    response.insert("status".to_string(), "healthy".to_string());
    response.insert("service".to_string(), "ore-ingest".to_string());
    Ok(Json(response))
}

/// Trigger ingest endpoint
async fn trigger_ingest(
    State(state): State<AppState>,
) -> Result<Json<HashMap<String, String>>, (StatusCode, Json<ErrorResponse>)> {
    // Get configuration from environment or from state
    let db_url = state.db_url;
    let rpc_url = std::env::var("SOLANA_RPC")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    // Create blockchain client
    let blockchain = BlockchainClient::new(&rpc_url);

    // Spawn ingestion task in the background
    let db_url_clone = db_url.clone();
    let rpc_url_clone = rpc_url.clone();

    tokio::spawn(async move {
        // Create new database connection for background task
        let db = match Database::new(&db_url_clone).await {
            Ok(db) => db,
            Err(e) => {
                eprintln!("❌ Failed to create database connection: {}", e);
                return;
            }
        };

        // Initialize schema
        if let Err(e) = db.init_schema().await {
            eprintln!("❌ Failed to initialize database schema: {}", e);
            return;
        }

        // Create ingest orchestrator and run
        let orchestrator = IngestOrchestrator::new(db, blockchain);
        if let Err(e) = orchestrator.run(&db_url_clone, &rpc_url_clone).await {
            eprintln!("❌ Ingestion failed: {}", e);
        }
    });

    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        "Ingestion started in background".to_string(),
    );
    response.insert("status".to_string(), "started".to_string());
    response.insert("database".to_string(), db_url);
    response.insert("rpc".to_string(), rpc_url);

    Ok(Json(response))
}

/// List rounds endpoint with pagination
async fn list_rounds(
    State(state): State<AppState>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<RoundListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let limit = params.limit;

    // Create database connection
    let db = match Database::new(&state.db_url).await {
        Ok(db) => db,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(&format!(
                    "Failed to connect to database: {}",
                    e
                ))),
            ))
        }
    };

    // Get rounds from database
    let rounds = match db.list_rounds(limit).await {
        Ok(rounds) => rounds,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(&format!(
                    "Failed to fetch rounds: {}",
                    e
                ))),
            ))
        }
    };

    // Get total count
    let total = match db.get_rounds_count().await {
        Ok(count) => count,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(&format!("Failed to get count: {}", e))),
            ))
        }
    };

    let response = RoundListResponse {
        rounds,
        total,
        limit: limit.unwrap_or(100),
    };

    Ok(Json(response))
}

/// Start the API server
pub async fn start_api_server(db_url: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState { db_url };

    let app = create_api_routes().with_state(app_state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("🚀 API server started on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Run the API server with graceful shutdown
pub async fn run_api_server(port: u16) -> anyhow::Result<()> {
    let db_url = std::env::var("TURSO_URL").unwrap_or_else(|_| "ore_rounds.db".to_string());
    let shutdown_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let shutdown_signal_clone = std::sync::Arc::clone(&shutdown_signal);

    // Set up signal handler
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            println!("\n🛑 Received shutdown signal");
            shutdown_signal_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            // Kill any processes holding the database
            kill_database_processes();
        }
    });

    // Initialize database schema
    let db = Database::new(&db_url).await?;
    db.init_schema().await?;

    // Start API server
    if let Err(e) = start_api_server(db_url, port).await {
        eprintln!("❌ Failed to start API server: {}", e);
        return Err(anyhow::anyhow!("API server failed: {}", e));
    }

    Ok(())
}

/// Kill any processes that might be holding the database files
fn kill_database_processes() {
    println!("🧹 Cleaning up database locks...");

    // Kill any ore-ingest processes first
    let _ = std::process::Command::new("pkill")
        .args(["-f", "ore-ingest"])
        .output();

    // Find and kill processes holding database files
    if let Ok(output) = std::process::Command::new("lsof")
        .args(["ore_rounds.db", "ore_rounds.db-wal", "ore_rounds.db-shm"])
        .output()
    {
        if output.status.success() {
            let lsof_output = String::from_utf8_lossy(&output.stdout);
            for line in lsof_output.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(pid) = parts[1].parse::<u32>() {
                        println!("Killing process {} holding database", pid);
                        let _ = std::process::Command::new("kill")
                            .args(["-9", &pid.to_string()])
                            .output();
                    }
                }
            }
        }
    }

    println!("✅ Database cleanup completed");
}
