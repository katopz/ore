use bincode;
use itertools::Itertools;
use ore_api::prelude::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

fn main() -> Result<(), anyhow::Error> {
    // Initialize RPC client
    let rpc = RpcClient::new_with_commitment(
        "https://api.mainnet-beta.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    // Get current round info to know what round we're in
    let board_pda = board_pda();
    let board_account = rpc.get_account(&board_pda.0)?;
    let board = bincode::deserialize::<Board>(&board_account.data)?;

    println!("Current Round Information:");
    println!("  Current Round ID: {}", board.round_id);
    println!("  Start Slot: {}", board.start_slot);
    println!("  End Slot: {}", board.end_slot);

    // Get current slot to see if current round is still active
    let clock_data = rpc.get_account_data(&solana_sdk::sysvar::clock::ID)?;
    let clock = bincode::deserialize::<solana_sdk::clock::Clock>(&clock_data)?;
    let current_slot = clock.slot;

    // Let's check what accounts exist for the program
    println!("Checking all program accounts...");
    let program_id = ore_api::id();
    let accounts = rpc.get_program_accounts(&program_id)?;
    println!(
        "Found {} total accounts for the ORE program",
        accounts.len()
    );

    // Show account size distribution first
    let mut size_counts = std::collections::HashMap::new();
    for (_, account) in &accounts {
        *size_counts.entry(account.data.len()).or_insert(0) += 1;
    }
    println!("Account size distribution:");
    for (size, count) in size_counts.iter().sorted() {
        println!("  {} bytes: {} accounts", size, count);
    }

    println!("\nKnown account sizes:");
    println!("  Board: {} bytes", std::mem::size_of::<Board>());
    println!("  Round: {} bytes", std::mem::size_of::<Round>());
    println!("  Config: {} bytes", std::mem::size_of::<Config>());
    println!("  Treasury: {} bytes", std::mem::size_of::<Treasury>());

    // Look specifically for round accounts
    let mut round_accounts = Vec::new();
    for (pubkey, account) in &accounts {
        if account.data.len() == std::mem::size_of::<Round>() {
            if let Ok(round) = bincode::deserialize::<Round>(&account.data) {
                round_accounts.push((pubkey, round));
            }
        }
    }

    println!("\nFound {} Round accounts", round_accounts.len());

    if !round_accounts.is_empty() {
        // Sort by round ID
        round_accounts.sort_by_key(|(_, round)| round.id);
        for (pubkey, round) in &round_accounts {
            println!("Round {}: {}", round.id, pubkey);

            // Get the random number and winning square
            if let Some(rng) = round.rng() {
                let winning_square_index = round.winning_square(rng);
                let is_split_reward = round.is_split_reward(rng);
                let did_hit_motherlode = round.did_hit_motherlode(rng);

                println!(
                    "  Winning Square: {} (row {}, col {})",
                    winning_square_index,
                    winning_square_index / 5 + 1,
                    winning_square_index % 5 + 1
                );
                println!("  Top Miner: {}", round.top_miner);
                println!("  Top Miner Reward: {} ORE", round.top_miner_reward);
                println!("  Split Reward: {}", is_split_reward);
                println!("  Motherlode Hit: {}", did_hit_motherlode);
                println!("  Total Deployed: {} SOL", round.total_deployed);
                println!("  Total Winnings: {} SOL", round.total_winnings);
                println!("  Expires At: {}", round.expires_at);
            } else {
                println!("  Not yet finalized (no slot hash)");
            }
            println!();
        }
    }

    println!("\n");
    if accounts
        .iter()
        .all(|(_, account)| account.data.len() != std::mem::size_of::<Round>())
    {
        println!("No round accounts found. They appear to be cleaned up after expiration.");
        println!("This is a cost-saving measure - once a round expires and claims are closed,");
        println!("the round account data is removed from the blockchain to save storage fees.");
        println!("\nTo find winner information, you need to check rounds BEFORE they expire.");
    }

    Ok(())
}
