use bincode;
use ore_api::prelude::*;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_sdk::commitment_config::CommitmentConfig;
use std::collections::HashMap;
use steel::AccountDeserialize;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize RPC client
    let rpc = RpcClient::new_with_commitment(
        "https://api.mainnet-beta.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    println!("=== ORE Round Winner Analysis ===\n");

    // Method 1: Check current board state
    println!("1. CURRENT BOARD STATE");
    let board_address = board_pda();
    let board_account = rpc.get_account(&board_address.0).await?;
    let board = bincode::deserialize::<Board>(&board_account.data)?;

    let clock_data = rpc.get_account_data(&solana_sdk::sysvar::clock::ID).await?;
    let clock = bincode::deserialize::<solana_sdk::clock::Clock>(&clock_data)?;
    let current_slot = clock.slot;

    println!("Current Round: {}", board.round_id);
    println!("Current Slot: {}", current_slot);
    println!("Round Start Slot: {}", board.start_slot);
    println!("Round End Slot: {}", board.end_slot);

    if current_slot >= board.end_slot {
        println!("Status: Round finished, waiting for reset");
    } else {
        let slots_remaining = board.end_slot - current_slot;
        let time_remaining = slots_remaining as f64 * 0.4;
        println!("Status: Active ({} seconds remaining)", time_remaining);
    }
    println!();

    // Method 2: Get all existing round accounts
    println!("2. EXISTING ROUND ACCOUNTS");
    let rounds = get_all_program_accounts::<Round>(&rpc).await?;
    println!(
        "Found {} round accounts that haven't been cleaned up yet",
        rounds.len()
    );

    if rounds.is_empty() {
        println!("❌ No round accounts found - they may have been cleaned up already");
        println!("   This is normal for cost optimization.");
    } else {
        // Sort by round ID (newest first)
        let mut sorted_rounds = rounds.clone();
        sorted_rounds.sort_by(|a, b| b.1.id.cmp(&a.1.id));

        for (i, (pubkey, round)) in sorted_rounds.iter().take(5).enumerate() {
            println!("\n{}. Round {} ({})", i + 1, round.id, pubkey);
            analyze_round(&round, current_slot);
        }
    }
    println!();

    // Method 3: Try to get recent rounds by checking sequential IDs
    println!("3. CHECKING RECENT ROUND IDS");
    let current_id = board.round_id;
    let mut found_rounds = Vec::new();

    // Check last 20 rounds
    for id in (0..=20u64).rev() {
        let check_id = current_id.saturating_sub(id);
        let round_pda = round_pda(check_id);

        match rpc.get_account(&round_pda.0).await {
            Ok(account) => {
                if account.data.len() == std::mem::size_of::<Round>() {
                    if let Ok(round) = bincode::deserialize::<Round>(&account.data) {
                        found_rounds.push((check_id, round));
                        println!("✓ Round {} found", check_id);
                    }
                }
            }
            Err(_) => {
                // Account doesn't exist or was cleaned up
            }
        }
    }

    if found_rounds.is_empty() {
        println!("❌ No recent rounds found via sequential check");
    } else {
        println!("\nFound {} recent rounds:", found_rounds.len());
        for (id, round) in &found_rounds {
            analyze_round(round, current_slot);
        }
    }
    println!();

    // Method 4: Check if there's any transaction log data
    println!("4. ALTERNATIVE DATA SOURCES");
    println!("Checking miner accounts for recent activity...");

    let miners = get_all_program_accounts::<Miner>(&rpc).await?;
    let mut active_rounds = HashMap::new();

    for (_, miner) in miners {
        if miner.round_id > 0 {
            *active_rounds.entry(miner.round_id).or_insert(0) += 1;
        }
    }

    println!("Active rounds found from miner data:");
    let mut active_vec: Vec<_> = active_rounds.iter().collect();
    active_vec.sort_by(|a, b| b.0.cmp(a.0));

    for (round_id, miner_count) in active_vec.iter().take(10) {
        println!("  Round {}: {} miners", round_id, miner_count);
    }
    println!();

    // Method 5: Try to estimate winner from entropy data if available
    println!("5. ENTROPY VARIABLE CHECK");
    let board_address = board_pda();
    let entropy_pda = entropy_api::state::var_pda(board_address.0, 0).0;

    match rpc.get_account(&entropy_pda).await {
        Ok(account) => {
            if let Ok(var) = entropy_api::state::Var::try_from_bytes(&account.data) {
                println!("✓ Found entropy variable:");
                println!("  Slot Hash: {:?}", var.slot_hash);
                println!("  Seed: {:?}", var.seed);
                println!("  Value: {:?}", var.value);

                if var.value != [0; 32] {
                    let rng_value = u64::from_le_bytes(var.value[0..8].try_into().unwrap())
                        ^ u64::from_le_bytes(var.value[8..16].try_into().unwrap())
                        ^ u64::from_le_bytes(var.value[16..24].try_into().unwrap())
                        ^ u64::from_le_bytes(var.value[24..32].try_into().unwrap());
                    let winning_square = (rng_value % 25) as usize;
                    println!(
                        "  Calculated Winning Square: {} (row {}, col {})",
                        winning_square,
                        winning_square / 5 + 1,
                        winning_square % 5 + 1
                    );
                }
            }
        }
        Err(_) => {
            println!("❌ No entropy variable found for current board");
        }
    }
    println!();

    // Summary
    println!("=== SUMMARY ===");
    if rounds.is_empty() {
        println!("🔍 Historical winner data appears to be cleaned up");
        println!("💡 To get winner information, you need to:");
        println!("   1. Check rounds BEFORE they expire");
        println!("   2. Monitor the blockchain in real-time");
        println!("   3. Use external services that store this data");
        println!("   4. Check transaction logs for reset events");

        println!("\n📡 Alternative approaches:");
        println!("   - Use Solana transaction logs to find Reset events");
        println!("   - Monitor the program's event emissions");
        println!("   - Use external indexers (like Solana Explorer)");

        println!("\n⏰ When to check for winners:");
        println!("   - Right after round.end_slot is reached");
        println!("   - Before the reset transaction is processed");
        println!("   - During the intermission period");
    } else {
        println!(
            "✅ Found {} historical rounds with winner data",
            rounds.len()
        );
        println!("💰 Use the data above to see winning squares and rewards");
    }

    Ok(())
}

async fn get_all_program_accounts<T>(
    rpc: &RpcClient,
) -> Result<Vec<(solana_sdk::pubkey::Pubkey, T)>, anyhow::Error>
where
    T: AccountDeserialize + steel::Discriminator + Clone,
{
    let discriminator = T::discriminator().to_le_bytes();
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
        if let Ok(data) = T::try_from_bytes(&account.data) {
            result.push((pubkey, data.clone()));
        }
    }
    Ok(result)
}

fn analyze_round(round: &Round, current_slot: u64) {
    println!("  Round ID: {}", round.id);

    // Check if round has winner data
    if let Some(rng) = round.rng() {
        let winning_square = round.winning_square(rng);
        let is_split = round.is_split_reward(rng);
        let motherlode_hit = round.did_hit_motherlode(rng);

        println!("  ✅ Winner Determined:");
        println!(
            "    Winning Square: {} (row {}, col {})",
            winning_square,
            winning_square / 5 + 1,
            winning_square % 5 + 1
        );
        println!("    Top Miner: {}", round.top_miner);
        println!("    Top Miner Reward: {} ORE", round.top_miner_reward);
        println!("    Split Reward: {}", is_split);
        println!("    Motherlode Hit: {}", motherlode_hit);
        if motherlode_hit {
            println!("    Motherlode Amount: {} ORE", round.motherlode);
        }
        println!("    Total Deployed: {} SOL", round.total_deployed);
        println!("    Total Winnings: {} SOL", round.total_winnings);
        println!("    Winners Count: {}", round.count[winning_square]);

        // Check expiration
        if current_slot >= round.expires_at {
            println!("  ⚠️  Round expired - cleanup may be pending");
        } else {
            let slots_until_expiry = round.expires_at - current_slot;
            let time_until_expiry = slots_until_expiry as f64 * 0.4;
            println!("  ⏰ Claims expire in: {:.2} seconds", time_until_expiry);
        }
    } else {
        println!("  ⏳ Not finalized yet (no slot hash)");
    }

    println!("  Round expires at: {}", round.expires_at);
    println!();
}
