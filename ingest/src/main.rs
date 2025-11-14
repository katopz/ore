use anyhow::Result;

#[cfg(feature = "api")]
mod api;
mod blockchain;
#[cfg(feature = "cli")]
mod cli;
mod database;
mod ingest;
mod types;

// clap only needed for CLI mode, not API mode

#[cfg(feature = "api")]
#[tokio::main]
async fn main() -> Result<()> {
    // API mode - use environment variables for configuration
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "4000".to_string())
        .parse()
        .unwrap_or(4000);

    api::run_api_server(port).await
}

#[cfg(feature = "cli")]
#[tokio::main]
async fn main() -> Result<()> {
    cli::run_ingestion().await
}

#[cfg(all(not(feature = "api"), not(feature = "cli")))]
compile_error!("Must specify either 'api' or 'cli' feature");

#[cfg(all(feature = "api", feature = "cli"))]
compile_error!("Cannot enable both 'api' and 'cli' features simultaneously");
