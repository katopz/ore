use anyhow::Result;

#[cfg(feature = "api")]
mod api;
mod blockchain;
mod database;
mod ingest;
mod types;

#[cfg(not(feature = "api"))]
mod cli;

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

    api::run_api_server(port).await
}

#[cfg(not(feature = "api"))]
/// Main function without API features
#[tokio::main]
async fn main() -> Result<()> {
    cli::run_ingestion().await
}
