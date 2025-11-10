# ORE Round Winner Data Ingestion

This project ingests historical ORE mining round winner data from Solana blockchain into a Turso SQLite database for analysis and tracking.

## 🎯 Purpose

The ORE protocol cleans up round accounts after they expire to save storage costs, making historical winner data difficult to access. This tool:

- **Preserves winner data** before it's cleaned up
- **Processes rounds efficiently** from blockchain
- **Resumes from last processed round** for reliability
- **Stores comprehensive winner information** in a queryable database
- **Modular architecture** with separated concerns

## 📊 Data Structure

Each round stores the following winner information:

- **Round ID**: Unique identifier (e.g., 48670, 48669, ...)
- **Winning Square**: Grid position (0-24, mapped to 5x5 board)
- **Winning Coordinates**: Row and column (1-5 each)
- **Top Miner**: Public key of the winning miner
- **Rewards**: ORE tokens awarded
- **SOL Metrics**: Total deployed, vaulted, and winnings
- **Game Mechanics**: Split rewards, motherlode hits
- **Expiration**: When claims close for the round

## 🚀 Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Navigate to project
cd ore/ingest
```

### Configuration

Set environment variables:

```bash
# Required: Solana RPC endpoint
export SOLANA_RPC="https://api.mainnet-beta.solana.com"

# Optional: Custom database file path
export TURSO_URL="ore_rounds.db"  # Default: local file
# export TURSO_URL="libsql://your-db.turso.io"  # Remote Turso
```

### Running

```bash
# Build and run (default features)
cargo run

# Run without API features (minimal)
cargo run --no-default-features

# Run with API features enabled (default)
cargo run --features api
```

The tool will:
1. **Initialize database schema** automatically
2. **Check existing data** for last processed round
3. **Find all available round accounts** from blockchain
4. **Process missing rounds** with rate limiting
5. **Store winner data** in database
6. **Resume on next run** if interrupted

## 📈 Usage Examples

### Basic Ingestion

```bash
# Process all missing rounds from latest down to round 1
cargo run

# Example output:
🏆 ORE Round Winner Data Ingestion
===================================
📊 Last processed round: 48660
📡 Using RPC: https://api.mainnet-beta.solana.com
💾 Database: ore_rounds.db
🔍 Finding all existing round accounts...
📊 Found 1070 existing round accounts
🎯 Highest round ID: 48670
🔄 Processing all rounds: 48670 to 1 (48670 total rounds)
✅ Processed round 48669
✅ Processed round 48668
🎉 Ingestion Complete!
📊 Processed: 9 rounds
❌ Errors: 0
💾 Database: ore_rounds.db
```

### Fresh Start

To start ingestion from scratch:

```bash
# Delete existing database
rm ore_rounds.db

# Or use a different database file
export TURSO_URL="fresh_ore.db"
cargo run
```

### Remote Database

```bash
# Set up remote Turso database
export TURSO_URL="libsql://your-db.turso.io?authToken=your-token"

# Run with remote storage
cargo run
```

## 🗄️ Database Schema

The SQLite database contains a single table `round_winners`:

```sql
CREATE TABLE round_winners (
    id INTEGER PRIMARY KEY,                    -- Round ID
    address TEXT NOT NULL,                     -- Round account address
    winning_square INTEGER NOT NULL,             -- Winning square (0-24)
    winning_row INTEGER NOT NULL,               -- Winning row (1-5)
    winning_col INTEGER NOT NULL,               -- Winning column (1-5)
    top_miner TEXT NOT NULL,                   -- Top miner pubkey
    top_miner_reward INTEGER NOT NULL,           -- ORE reward amount
    split_reward BOOLEAN NOT NULL,              -- Whether rewards are split
    motherlode_hit BOOLEAN NOT NULL,            -- Whether motherlode was hit
    motherlode_amount INTEGER NOT NULL,          -- Motherlode amount if hit
    total_deployed INTEGER NOT NULL,             -- Total SOL deployed
    total_vaulted INTEGER NOT NULL,             -- Total SOL vaulted
    total_winnings INTEGER NOT NULL,             -- Total SOL won
    winners_count INTEGER NOT NULL,              -- Number of winners
    expires_at INTEGER NOT NULL,                -- Expiration slot
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP -- When recorded
);

-- Index for faster queries
CREATE INDEX idx_round_winners_id ON round_winners(id);
```

## 🔍 Querying the Data

### Example Queries

```sql
-- Find most recent rounds
SELECT id, winning_square, top_miner, winners_count 
FROM round_winners ORDER BY id DESC LIMIT 10;

-- Find rounds with motherlode hits
SELECT id, motherlode_amount, winners_count 
FROM round_winners 
WHERE motherlode_hit = TRUE 
ORDER BY id DESC;

-- Analyze winning square distribution
SELECT winning_square, COUNT(*) as frequency
FROM round_winners 
GROUP BY winning_square 
ORDER BY frequency DESC;

-- Find highest reward rounds
SELECT id, top_miner_reward, total_deployed
FROM round_winners 
ORDER BY top_miner_reward DESC 
LIMIT 10;

-- Get rounds with split rewards
SELECT id, winning_square, winners_count
FROM round_winners 
WHERE split_reward = TRUE 
ORDER BY id DESC;

-- Check database statistics
SELECT 
    COUNT(*) as total_rounds,
    MIN(id) as earliest_round,
    MAX(id) as latest_round,
    AVG(winners_count) as avg_winners
FROM round_winners;
```

## ⚙️ Technical Details

### Winner Determination

The winning square is determined using entropy from Solana's slot hash:

1. **Slot Hash**: 32-byte hash from completed round
2. **RNG Calculation**: XOR operation on 8-byte chunks
3. **Winning Square**: `rng % 25` (maps to 5x5 grid)
4. **Coordinates**: Row = (square ÷ 5) + 1, Column = (square % 5) + 1

### Rate Limiting

- **200ms delay** between round requests
- **Sequential processing** to avoid API limits
- **Error recovery** with 1s backoff
- **Clean logging** showing only processed rounds

### Database Resumability

- **Automatic detection** of last processed round
- **Idempotent inserts** with PRIMARY KEY constraints
- **Graceful handling** of missing/cleaned rounds
- **Progress tracking** with processed counts

## 🛠️ Development

### Project Structure

```
ingest/
├── Cargo.toml          # Dependencies and features
├── src/
│   ├── main.rs        # Main entry point and routing
│   ├── types.rs       # Data structures
│   ├── database.rs    # Database operations
│   ├── blockchain.rs  # Solana interactions
│   ├── api.rs         # API endpoints (feature-gated)
│   └── ingest.rs      # Business logic orchestrator
├── examples/           # Example scripts
│   ├── find_round_winners.rs
│   ├── get_round.rs
│   └── get_round_winner.rs
└── README.md           # This file
```

### Module Architecture

- **types.rs**: All data structures and API models
- **database.rs**: SQLite/Turso operations with connection management
- **blockchain.rs**: Solana RPC client and round account processing
- **main.rs**: Application entry point with feature-gated routing
- **api.rs**: HTTP API endpoints (when `api` feature enabled)

### Dependencies

```toml
[dependencies]
ore-api = { path = "../api" }          # ORE program bindings
solana-client = "^2.1"                 # Solana RPC client
turso = "0.2.2"                         # LibSQL/Turso client
tokio = { version = "1.37.0", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"                         # Error handling

# Optional dependencies (feature-gated)
axum = { version = "0.8.4", optional = true }        # HTTP framework
clap = { version = "4.0", features = ["derive"], optional = true }  # CLI parsing
tower-http = { version = "0.6", features = ["cors"], optional = true }  # CORS

[features]
default = ["api"]    # API enabled by default
api = ["dep:axum", "dep:clap", "dep:tower-http"]
```

### Building

```bash
# Standard build (with API features)
cargo build

# Minimal build (no API)
cargo build --no-default-features

# Release build
cargo build --release
```

### Testing

```bash
# Test with local database
export TURSO_URL="test.db"
cargo run

# Test with different RPC
export SOLANA_RPC="https://api.devnet.solana.com"
cargo run

# Run tests
cargo test
```

## 📝 Notes & Limitations

### Data Availability

- Round accounts are **cleaned up after expiration** (typically ~24 hours)
- **Older rounds may be unavailable** if not captured in time
- **Current rounds** don't have winner data until finalized
- **Historical preservation** requires regular ingestion runs

### API Considerations

- **Rate limiting** is built-in to avoid RPC limits
- **Sequential processing** ensures reliability over speed
- **Error handling** retries failed requests with backoff
- **Connection pooling** and timeout management

### Database Size & Performance

- Each round record: ~200 bytes
- 100,000 rounds: ~20MB database file
- Indexes add ~30% overhead for faster queries
- Suitable for both local SQLite and remote Turso
- Resumable ingestion saves bandwidth and time

## 🔗 Related Tools

- **examples/**: Real-time round analysis scripts
- **ORE CLI**: Official command-line interface
- **Solana Explorer**: Transaction history lookup
- **ORE Dashboard**: Mining statistics visualization

## 📄 License

Apache-2.0 License - See parent project for details.