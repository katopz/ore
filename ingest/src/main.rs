use anyhow::Result;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod api;
mod blockchain;
mod database;
mod ingest;
mod types;

use api::AppState;
use blockchain::BlockchainClient;
use database::Database;

// clap only needed for CLI mode, not API mode

#[cfg(feature = "api")]
/// Main function with API features enabled
#[tokio::main]
async fn main() -> Result<()> {
    // API mode - use environment variables for configuration
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "4000".to_string())
        .parse()
        .unwrap_or(4000);

    run_api_server(port).await
}

#[cfg(not(feature = "api"))]
/// Main function without API features
#[tokio::main]
async fn main() -> Result<()> {
    run_ingestion().await
}

#[cfg(feature = "api")]
/// Run the API server
async fn run_api_server(port: u16) -> Result<()> {
    let db_url = std::env::var("TURSO_URL").unwrap_or_else(|_| "/app/data/ore.db".to_string());

    // Initialize database schema
    let db = Database::new(&db_url).await?;
    db.init_schema().await?;

    // Create app state
    let app_state = AppState { db_url };

    // Create API routes
    let app = api::create_api_routes().with_state(app_state).layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    );

    // Bind to port
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("🚀 API server started on http://0.0.0.0:{}", port);

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}

/// Run the ingestion process
async fn run_ingestion() -> Result<()> {
    let db_url = std::env::var("TURSO_URL").unwrap_or_else(|_| "/app/data/ore.db".to_string());
    let rpc_url = std::env::var("SOLANA_RPC")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    // Initialize database
    let db = Database::new(&db_url).await?;
    let blockchain = BlockchainClient::new(&rpc_url);

    // Create ingest orchestrator and run
    let orchestrator = ingest::IngestOrchestrator::new(db, blockchain);
    orchestrator.run(&db_url, &rpc_url).await?;

    Ok(())
}
