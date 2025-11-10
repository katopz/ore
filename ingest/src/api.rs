#[cfg(feature = "api")]
use crate::blockchain::BlockchainClient;
#[cfg(feature = "api")]
use crate::database::Database;
#[cfg(feature = "api")]
use crate::ingest::IngestOrchestrator;
#[cfg(feature = "api")]
use crate::types::{ErrorResponse, RoundListResponse};
#[cfg(feature = "api")]
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
#[cfg(feature = "api")]
use serde::Deserialize;
#[cfg(feature = "api")]
use std::collections::HashMap;
#[cfg(feature = "api")]
use tokio::net::TcpListener;
#[cfg(feature = "api")]
use tower_http::cors::{Any, CorsLayer};

#[cfg(feature = "api")]
#[derive(Debug, Deserialize)]
pub struct ListQueryParams {
    pub limit: Option<i64>,
}

#[cfg(feature = "api")]
/// API server state
#[derive(Clone)]
pub struct AppState {
    pub db_url: String,
}

#[cfg(feature = "api")]
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

#[cfg(feature = "api")]
/// Health check endpoint
async fn health_check() -> Result<Json<HashMap<String, String>>, StatusCode> {
    let mut response = HashMap::new();
    response.insert("status".to_string(), "healthy".to_string());
    response.insert("service".to_string(), "ore-ingest".to_string());
    Ok(Json(response))
}

#[cfg(feature = "api")]
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

    // Spawn the ingestion task in the background
    let db_url_clone = db_url.clone();
    let rpc_url_clone = rpc_url.clone();

    tokio::spawn(async move {
        // Create new database connection for the background task
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

#[cfg(feature = "api")]
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

#[cfg(feature = "api")]
/// Start the API server
pub async fn start_api_server(db_url: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState { db_url };

    let app = create_api_routes().with_state(app_state);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("🚀 API server started on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(not(feature = "api"))]
/// No-op function when API feature is not enabled
pub async fn start_api_server(
    _db: crate::database::Database,
    _port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("❌ API feature not enabled. Run with --features api to enable API server.");
    Ok(())
}
