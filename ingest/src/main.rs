use anyhow::Result;
#[cfg(feature = "api")]
mod api;
mod blockchain;
#[cfg(feature = "cli")]
mod cli;
mod database;
mod ingest;
mod types;

#[cfg(feature = "api")]
/// Main function with API features enabled
#[tokio::main]
async fn main() -> Result<()> {
    // Use environment variable for port, fallback to 8080
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);

    crate::api::run_api_server(port).await
}

#[cfg(feature = "cli")]
/// Main function with CLI features enabled
#[tokio::main]
async fn main() -> Result<()> {
    cli::run_ingestion().await
}
