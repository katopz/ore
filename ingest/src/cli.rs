use crate::blockchain::BlockchainClient;
use crate::database::Database;
use crate::ingest::IngestOrchestrator;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;

/// Kill any processes that might be holding the database files
fn kill_database_processes() {
    println!("🧹 Cleaning up database locks...");

    // Kill any ore-ingest processes first
    let _ = std::process::Command::new("pkill")
        .args(&["-f", "ore-ingest"])
        .output();

    // Find and kill processes holding database files
    if let Ok(output) = std::process::Command::new("lsof")
        .args(&["ore_rounds.db", "ore_rounds.db-wal", "ore_rounds.db-shm"])
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
                            .args(&["-9", &pid.to_string()])
                            .output();
                    }
                }
            }
        }
    }

    println!("✅ Database cleanup completed");
}

/// Run the ingestion process with graceful shutdown
pub async fn run_ingestion() -> Result<()> {
    let db_url = std::env::var("TURSO_URL").unwrap_or_else(|_| "ore.db".to_string());
    let rpc_url = std::env::var("SOLANA_RPC")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let shutdown_signal_clone = Arc::clone(&shutdown_signal);

    // Set up signal handler
    tokio::spawn(async move {
        if signal::ctrl_c().await.is_ok() {
            println!("\n🛑 Received shutdown signal");
            shutdown_signal_clone.store(true, Ordering::SeqCst);
            // Kill any processes holding the database
            kill_database_processes();
        }
    });

    // Initialize database
    let db = Database::new(&db_url).await?;
    let blockchain = BlockchainClient::new(&rpc_url);

    // Create ingest orchestrator and run
    let orchestrator = IngestOrchestrator::new(db, blockchain);
    orchestrator.run(&db_url, &rpc_url).await?;

    Ok(())
}
