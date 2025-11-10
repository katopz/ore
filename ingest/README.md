# ORE Round Winner Data Ingestion

This project ingests historical ORE mining round winner data from the Solana blockchain into a Turso SQLite database for analysis and tracking.

## 🎯 Purpose

The ORE protocol cleans up round accounts after they expire to save storage costs, making historical winner data difficult to access. This tool:

- **Preserves winner data** before it's cleaned up
- **Processes rounds sequentially** to avoid API limits
- **Resumes from last processed round** for reliability
- **Stores comprehensive winner information** in a queryable database

## 📊 Data Structure

Each round stores the following winner information:

- **Round ID**: Unique identifier (1, 2, 3, ...)
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

# Optional: Custom Turso database URL
export TURSO_URL="ore_rounds.db"  # Default: local file
# export TURSO_URL="libsql://your-db.turso.io"  # Remote Turso
```

### Running

```bash
# Build and run
cargo run

# The tool will:
# 1. Check existing database for last processed round
# 2. Get current board round
# 3. Process missing rounds one-by-one
# 4. Store winner data in database
# 5. Resume on next run if interrupted
```

## 📈 Usage Examples

### Basic Ingestion

```bash
# Process all missing rounds from latest to round 1
cargo run

# Example output:
🏆 ORE Round Winner Data Ingestion
===================================
📊 Last processed round: 48547
🎯 Current board round: 105
🔄 Processing rounds 48548 to 105 (48443 total rounds)
✅ Processed 10 rounds...
✅ Processed 20 rounds...
🎉 Ingestion Complete!
📊 Processed: 48443 rounds
❌ Errors: 0
💾 Database: ore_rounds.db
```

### Resume from Specific Round

The tool automatically resumes from the last processed round. To start fresh:

```bash
# Delete database to start from round 1
rm ore_rounds.db

# Or specify empty database
export TURSO_URL="fresh_ore.db"
```

### Using Remote Turso

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
```

## 🔍 Querying the Data

### Example Queries

```sql
-- Find most recent rounds
SELECT * FROM round_winners ORDER BY id DESC LIMIT 10;

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
- **Progress reporting** every 10 rounds

### Database Resumability

- **Automatic detection** of last processed round
- **Idempotent inserts** with PRIMARY KEY constraints
- **Graceful handling** of missing/cleaned rounds
- **Progress tracking** with processed counts

## 🛠️ Development

### Project Structure

```
ingest/
├── Cargo.toml          # Dependencies and metadata
├── src/
│   └── main.rs        # Main ingestion logic
├── sql/
│   └── schema.sql      # Database schema
└── README.md           # This file
```

### Dependencies

- `ore-api`: ORE program API bindings
- `solana-client`: Solana RPC client
- `turso`: LibSQL/Turso database client
- `tokio`: Async runtime
- `chrono`: Date/time handling
- `serde`: Serialization/deserialization

### Building

```bash
cargo build --release
```

### Testing

```bash
# Test with local database
export TURSO_URL="test.db"
cargo run

# Test with mainnet RPC
export SOLANA_RPC="https://api.mainnet-beta.solana.com"
cargo run
```

## 📝 Notes & Limitations

### Data Availability

- Round accounts are **cleaned up after expiration** (typically ~24 hours)
- **Older rounds may be unavailable** if not captured in time
- **Current rounds** don't have winner data until finalized

### API Considerations

- **Rate limiting** is built-in to avoid RPC limits
- **Sequential processing** ensures reliability over speed
- **Error handling** retries failed requests
- **Large range queries** may time out on some RPCs

### Database Size

- Each round record: ~200 bytes
- 100,000 rounds: ~20MB database file
- Indexes add ~30% overhead
- Suitable for SQLite and Turso

## 🔗 Related Tools

- `find_round_winners.rs`: Real-time round analysis
- ORE CLI: Official command-line interface
- Solana Explorer: Transaction history lookup
- ORE Dashboard: Mining statistics

## 📄 License

Apache-2.0 License - See parent project for details.