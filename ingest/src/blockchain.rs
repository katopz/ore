use crate::types::RoundWinner;
use anyhow::Result;
use chrono::Utc;
use ore_api::prelude::*;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_sdk::commitment_config::CommitmentConfig;
use steel::{AccountDeserialize, Discriminator};
use tokio::time::{sleep, Duration};

/// Blockchain operations module
pub struct BlockchainClient {
    pub rpc: RpcClient,
}

impl BlockchainClient {
    /// Initialize RPC client
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc: RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed()),
        }
    }

    /// Get all existing round accounts from the program
    pub async fn get_all_round_accounts(&self) -> Result<Vec<(solana_sdk::pubkey::Pubkey, Round)>> {
        let discriminator = Round::discriminator().to_le_bytes();
        let accounts = self
            .rpc
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
                result.push((pubkey, *data));
            }
        }
        Ok(result)
    }

    /// Process a round from existing account data
    pub async fn process_round_from_data(
        &self,
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

    /// Rate limiting between requests
    pub async fn delay_between_requests() {
        sleep(Duration::from_millis(200)).await;
    }
}
