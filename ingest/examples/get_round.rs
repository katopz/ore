use bincode;
use ore_api::prelude::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

fn main() -> Result<(), anyhow::Error> {
    // Initialize RPC client (you can use any RPC endpoint)
    let rpc = RpcClient::new_with_commitment(
        "https://api.mainnet-beta.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    // Get the board account which contains the current round information
    let board_pda = board_pda();
    let account = rpc.get_account(&board_pda.0)?;

    // Deserialize the board data
    let board = bincode::deserialize::<Board>(&account.data)?;

    // Print the current round information
    println!("Current Round Information:");
    println!("  Round ID: {}", board.round_id);
    println!("  Start Slot: {}", board.start_slot);
    println!("  End Slot: {}", board.end_slot);

    // Get current slot to calculate time remaining
    let clock_data = rpc.get_account_data(&solana_sdk::sysvar::clock::ID)?;
    let clock = bincode::deserialize::<solana_sdk::clock::Clock>(&clock_data)?;
    let current_slot = clock.slot;

    if board.end_slot > current_slot {
        let slots_remaining = board.end_slot - current_slot;
        let seconds_remaining = slots_remaining as f64 * 0.4; // ~400ms per slot
        println!("  Time Remaining: {:.2} seconds", seconds_remaining);
    } else {
        println!("  Round has ended");
    }

    Ok(())
}
