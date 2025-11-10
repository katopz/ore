use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

/// API response for list endpoint
#[derive(Debug, Serialize)]
pub struct RoundListResponse {
    pub rounds: Vec<RoundWinner>,
    pub total: i64,
    pub limit: i64,
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ErrorResponse {
    pub fn new(message: &str) -> Self {
        Self {
            error: message.to_string(),
        }
    }
}
