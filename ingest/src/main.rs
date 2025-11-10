use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use ore_api::prelude::*;
use serde::{Deserialize, Serialize};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_sdk::commitment_config::CommitmentConfig;
use steel::{AccountDeserialize, Discriminator};
use tokio::time::{sleep, Duration};
use turso::{Builder, Connection};

/// Database schema for round winner data
#[derive(Debug, Serialize, Deserialize)]
pub struct RoundWinner {
    pub id: i64,                   // Round ID
    pub address: String,           // Round account address
    pub winning_square: i64,       // 0-24
    pub winning_row: i64,          // 1-5
    pub winning_col: i64,          // 1-5
    pub top_miner: String,         // Top miner pubkey
    pub top_miner_reward: i64,     // ORE reward
    pub split_reward: bool,        // Whether rewards are split
    pub motherlode_hit: bool,      // Whether motherlode was hit
    pub motherlode_amount: i64,    // Motherlode amount if hit
    pub total_deployed: i64,       // Total SOL deployed
    pub total_vaulted: i64,        // Total SOL vaulted
    pub total_winnings: i64,       // Total SOL won
    pub winners_count: i64,        // Number of winners
    pub expires_at: i64,           // Expiration slot
    pub created_at: DateTime<Utc>, // When we recorded this
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🏆 ORE Round Winner Data Ingestion");
    println!("===================================");

    // Initialize database
    let db_url = std::env::var("TURSO_URL").unwrap_or_else(|_| "ore_rounds.db".to_string());
    let db = Builder::new_local(&db_url)
        .build()
        .await
        .context("Failed to initialize Turso database")?;

    let conn = db
        .connect()
        .context("Failed to connect to Turso database")?;

    init_schema(&conn)
        .await
        .context("Failed to initialize database schema")?;

    // Check existing data to resume from last processed round
    let last_round_id = get_last_round_id(&conn)
        .await
        .context("Failed to get last processed round ID")?;

    println!(
        "📊 Last processed round: {}",
        last_round_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "None".to_string())
    );

    // Initialize RPC client
    let rpc_url = std::env::var("SOLANA_RPC")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let rpc = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());
    println!("📡 Using RPC: {}", rpc_url);

    println!("📡 Using RPC: {}", rpc_url);
    println!("💾 Database: {}", db_url);

    // Get all existing round accounts to find the actual highest round ID
    println!("🔍 Finding all existing round accounts...");
    let existing_rounds = get_all_round_accounts(&rpc)
        .await
        .context("Failed to get existing round accounts")?;

    if existing_rounds.is_empty() {
        println!("❌ No round accounts found - they may have been cleaned up already");
        println!("   This is normal for cost optimization.");
        return Ok(());
    }

    // Find the highest round ID
    let highest_round_id = existing_rounds
        .iter()
        .map(|(_, round)| round.id)
        .max()
        .unwrap_or(0);

    println!("📊 Found {} existing round accounts", existing_rounds.len());
    println!("🎯 Highest round ID: {}", highest_round_id);

    // Determine starting point - process all rounds from highest down to round 1
    let start_round = highest_round_id;
    let end_round = 1; // Process until round 1

    // Check if we already processed these
    let last_processed = last_round_id.unwrap_or(0);
    if last_processed >= start_round as i64 {
        println!(
            "✅ All recent rounds already processed! Latest: {}, Highest found: {}",
            last_processed, start_round
        );
        return Ok(());
    }

    // Adjust end_round to not duplicate processed rounds
    let end_round = end_round.max((last_processed + 1) as u64);

    println!(
        "🔄 Processing all rounds: {} to {} ({} total rounds)",
        start_round,
        end_round,
        start_round - end_round + 1
    );

    // Process the existing round accounts we found
    let mut processed_count = 0;
    let mut error_count = 0;

    // Sort rounds by ID (newest first) and process all
    let mut sorted_rounds: Vec<_> = existing_rounds.iter().collect();
    sorted_rounds.sort_by(|a, b| b.1.id.cmp(&a.1.id));

    for (pubkey, round) in sorted_rounds.iter() {
        let round_id = round.id;

        // Skip if already processed
        if round_exists(&conn, round_id as i64).await? {
            println!("⏭️  Round {} already processed", round_id);
            continue;
        }

        match process_round_from_data(&conn, pubkey, round).await {
            Ok(Some(winner_data)) => {
                if let Err(e) = save_round_winner(&conn, &winner_data).await {
                    println!("❌ Failed to save round {}: {}", round_id, e);
                    error_count += 1;
                } else {
                    processed_count += 1;
                    if processed_count % 100 == 0 {
                        println!("✅ Processed {} rounds...", processed_count);
                    }
                }
            }
            Ok(None) => {
                // Round not finalized yet
                println!("⏭️  Round {} not finalized yet", round_id);
            }
            Err(e) => {
                println!("❌ Error processing round {}: {}", round_id, e);
                error_count += 1;
            }
        }

        // Rate limiting between requests
        sleep(Duration::from_millis(200)).await;
    }

    println!("\n🎉 Ingestion Complete!");
    println!("📊 Processed: {} rounds", processed_count);
    println!("❌ Errors: {}", error_count);
    println!("💾 Database: {}", db_url);

    Ok(())
}

/// Initialize database schema
async fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS round_winners (
            id INTEGER PRIMARY KEY,
            address TEXT NOT NULL,
            winning_square INTEGER NOT NULL,
            winning_row INTEGER NOT NULL,
            winning_col INTEGER NOT NULL,
            top_miner TEXT NOT NULL,
            top_miner_reward INTEGER NOT NULL,
            split_reward BOOLEAN NOT NULL,
            motherlode_hit BOOLEAN NOT NULL,
            motherlode_amount INTEGER NOT NULL,
            total_deployed INTEGER NOT NULL,
            total_vaulted INTEGER NOT NULL,
            total_winnings INTEGER NOT NULL,
            winners_count INTEGER NOT NULL,
            expires_at INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"#,
        (),
    )
    .await?;

    // Create index for faster queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_round_winners_id ON round_winners(id)",
        (),
    )
    .await?;

    Ok(())
}

/// Get the last processed round ID from database
async fn get_last_round_id(conn: &Connection) -> Result<Option<i64>> {
    let mut rows = conn
        .query("SELECT id FROM round_winners ORDER BY id DESC LIMIT 1", ())
        .await?;

    if let Some(row) = rows.next().await? {
        let id: i64 = row.get(0)?;
        Ok(Some(id))
    } else {
        Ok(None)
    }
}

/// Get current board information

/// Process a round from existing account data
async fn process_round_from_data(
    _conn: &Connection,
    pubkey: &solana_sdk::pubkey::Pubkey,
    round: &Round,
) -> Result<Option<RoundWinner>> {
    // Extract winner information
    let winner_info = if let Some(rng) = round.rng() {
        let winning_square = round.winning_square(rng) as i64;
        Some(RoundWinner {
            id: round.id as i64,
            address: pubkey.to_string(),
            winning_square,
            winning_row: (winning_square / 5) + 1,
            winning_col: (winning_square % 5) + 1,
            top_miner: round.top_miner.to_string(),
            top_miner_reward: round.top_miner_reward as i64,
            split_reward: round.is_split_reward(rng),
            motherlode_hit: round.did_hit_motherlode(rng),
            motherlode_amount: round.motherlode as i64,
            total_deployed: round.total_deployed as i64,
            total_vaulted: round.total_vaulted as i64,
            total_winnings: round.total_winnings as i64,
            winners_count: round.count[round.winning_square(rng)] as i64,
            expires_at: round.expires_at as i64,
            created_at: Utc::now(),
        })
    } else {
        // Round not finalized yet
        None
    };

    Ok(winner_info)
}

/// Check if round already exists in database
async fn round_exists(conn: &Connection, round_id: i64) -> Result<bool> {
    let mut rows = conn
        .query("SELECT id FROM round_winners WHERE id = ?", [round_id])
        .await?;

    Ok(rows.next().await?.is_some())
}

/// Save round winner data to database
async fn save_round_winner(conn: &Connection, winner: &RoundWinner) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO round_winners (
            id, address, winning_square, winning_row, winning_col,
            top_miner, top_miner_reward, split_reward, motherlode_hit,
            motherlode_amount, total_deployed, total_vaulted, total_winnings,
            winners_count, expires_at, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        (
            winner.id,
            winner.address.as_str(),
            winner.winning_square,
            winner.winning_row,
            winner.winning_col,
            winner.top_miner.as_str(),
            winner.top_miner_reward,
            winner.split_reward,
            winner.motherlode_hit,
            winner.motherlode_amount,
            winner.total_deployed,
            winner.total_vaulted,
            winner.total_winnings,
            winner.winners_count,
            winner.expires_at,
            winner.created_at.to_rfc3339(),
        ),
    )
    .await?;

    Ok(())
}

/// Get all existing round accounts from the program
async fn get_all_round_accounts(
    rpc: &RpcClient,
) -> Result<Vec<(solana_sdk::pubkey::Pubkey, Round)>> {
    let discriminator = Round::discriminator().to_le_bytes();
    let accounts = rpc
        .get_program_accounts_with_config(
            &ore_api::ID,
            RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
                    0,
                    &discriminator,
                ))]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await?;

    let mut result = Vec::new();
    for (pubkey, account) in accounts {
        if let Ok(data) = Round::try_from_bytes(&account.data) {
            result.push((pubkey, data.clone()));
        }
    }
    Ok(result)
}
