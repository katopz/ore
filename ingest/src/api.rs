#[cfg(feature = "api")]
use crate::database::Database;
#[cfg(feature = "api")]
use crate::types::{ErrorResponse, RoundListResponse};
#[cfg(feature = "api")]
use axum::{
    extract::{Path, Query, State},
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
pub struct AppState {
    pub db: Database,
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
    // For now, return a message indicating manual ingest is required
    // In a real implementation, you might trigger a background job
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        "Ingest should be run manually via CLI".to_string(),
    );
    response.insert(
        "command".to_string(),
        "cargo run --bin ore-ingest".to_string(),
    );
    Ok(Json(response))
}

#[cfg(feature = "api")]
/// List rounds endpoint with pagination
async fn list_rounds(
    State(state): State<AppState>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<RoundListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let limit = params.limit;

    // Get rounds from database
    let rounds = match state.db.list_rounds(limit).await {
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
    let total = match state.db.get_rounds_count().await {
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
pub async fn start_api_server(db: Database, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState { db };

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
