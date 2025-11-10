use anyhow::Result;

mod api;
mod blockchain;
mod database;
mod ingest;
mod types;

use blockchain::BlockchainClient;
use database::Database;

#[cfg(feature = "api")]
use clap::{Arg, Command};

#[cfg(feature = "api")]
/// Main function with API features enabled
#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("ore-ingest")
        .version("0.1.0")
        .about("ORE round winner data ingestion service")
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_name("MODE")
                .help("Run mode: 'ingest' or 'api'")
                .default_value("ingest"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Port for API server (only used in api mode)")
                .default_value("8080"),
        )
        .get_matches();

    let mode = matches.get_one::<String>("mode").unwrap();
    let port: u16 = matches.get_one::<String>("port").unwrap().parse()?;

    match mode.as_str() {
        "api" => run_api_server(port).await,
        "ingest" => run_ingestion().await,
        _ => {
            eprintln!("Invalid mode. Use 'ingest' or 'api'");
            Ok(())
        }
    }
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
    let db_url = std::env::var("TURSO_URL").unwrap_or_else(|_| "ore_rounds.db".to_string());

    // Initialize database schema
    let db = Database::new(&db_url).await?;
    db.init_schema().await?;

    // Start API server
    if let Err(e) = api::start_api_server(db_url, port).await {
        eprintln!("❌ Failed to start API server: {}", e);
        return Err(anyhow::anyhow!("API server failed: {}", e));
    }

    Ok(())
}

/// Run the ingestion process
async fn run_ingestion() -> Result<()> {
    let db_url = std::env::var("TURSO_URL").unwrap_or_else(|_| "ore_rounds.db".to_string());
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
