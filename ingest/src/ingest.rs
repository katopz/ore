use anyhow::{Context, Result};
use tokio::time::Duration;

use crate::blockchain::BlockchainClient;
use crate::database::Database;

/// Main ingest orchestrator
pub struct IngestOrchestrator {
    pub db: Database,
    pub blockchain: BlockchainClient,
}

impl IngestOrchestrator {
    /// Create new ingest orchestrator
    pub fn new(db: Database, blockchain: BlockchainClient) -> Self {
        Self { db, blockchain }
    }

    /// Run the full ingestion process
    pub async fn run(&self, db_url: &str, rpc_url: &str) -> Result<()> {
        println!("🏆 ORE Round Winner Data Ingestion");
        println!("===================================");

        // Initialize database schema
        self.db
            .init_schema()
            .await
            .context("Failed to initialize database schema")?;

        // Get last processed round to resume
        let last_round_id = self
            .db
            .get_last_round_id()
            .await
            .context("Failed to get last processed round ID")?;

        println!(
            "📊 Last processed round: {}",
            last_round_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "None".to_string())
        );

        println!("📡 Using RPC: {}", rpc_url);
        println!("💾 Database: {}", db_url);

        // Get all existing round accounts
        println!("🔍 Finding all existing round accounts...");
        let existing_rounds = self
            .blockchain
            .get_all_round_accounts()
            .await
            .context("Failed to get existing round accounts")?;

        if existing_rounds.is_empty() {
            println!("❌ No round accounts found - they may have been cleaned up already");
            return Ok(());
        }

        // Find highest round and determine processing range
        let highest_round_id = existing_rounds
            .iter()
            .map(|(_, round)| round.id)
            .max()
            .unwrap_or(0);

        println!("📊 Found {} existing round accounts", existing_rounds.len());
        println!("🎯 Highest round ID: {}", highest_round_id);

        // Process all rounds from highest down to round 1
        let start_round = highest_round_id;
        let end_round = 1;

        // Check if we already processed all rounds
        let last_processed = last_round_id.unwrap_or(0);
        if last_processed >= start_round as i64 {
            println!(
                "✅ All recent rounds already processed! Latest: {}, Highest found: {}",
                last_processed, start_round
            );
            return Ok(());
        }

        println!(
            "🔄 Processing all rounds: {} to {} ({} total rounds)",
            start_round,
            end_round,
            start_round - end_round + 1
        );

        // Process rounds
        let mut processed_count = 0;
        let mut error_count = 0;

        // Sort rounds by ID (newest first) and process all
        let mut sorted_rounds: Vec<_> = existing_rounds.iter().collect();
        sorted_rounds.sort_by(|a, b| b.1.id.cmp(&a.1.id));

        for (pubkey, round) in sorted_rounds.iter() {
            let round_id = round.id;

            // Skip if already processed
            if self.db.round_exists(round_id as i64).await? {
                continue;
            }

            match self.blockchain.process_round_from_data(pubkey, round).await {
                Ok(Some(winner_data)) => {
                    if let Err(e) = self.db.save_round_winner(&winner_data).await {
                        println!("❌ Failed to save round {}: {}", round_id, e);
                        error_count += 1;
                    } else {
                        processed_count += 1;
                        println!("✅ Processed round {}", round_id);
                    }
                }
                Ok(None) => {
                    // Round not finalized yet - don't log to reduce verbosity
                }
                Err(e) => {
                    println!("❌ Error processing round {}: {}", round_id, e);
                    error_count += 1;

                    // Rate limiting - wait on errors
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }

            // Rate limiting between requests
            BlockchainClient::delay_between_requests().await;
        }

        println!("\n🎉 Ingestion Complete!");
        println!("📊 Processed: {} rounds", processed_count);
        println!("❌ Errors: {}", error_count);
        println!("💾 Database: {}", db_url);

        Ok(())
    }
}
