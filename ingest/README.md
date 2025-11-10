# ORE Ingest Service

A high-performance data ingestion service for ORE (Solana-native store of value) that fetches round winner data from the Solana blockchain and provides both CLI and REST API interfaces for data management.

## 🎯 Features

- **Dual Mode Operation**: Run as CLI tool or API server
- **Background Ingestion**: Non-blocking data processing via API
- **RESTful API**: HTTP endpoints for triggering and querying data
- **Modular Architecture**: Clean separation of concerns
- **Database Storage**: Turso/SQLite backend with automatic schema
- **Resumable Processing**: Automatic resume from last processed round

## 🚀 Quick Start

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
# Solana RPC endpoint (optional, has default)
export SOLANA_RPC="https://api.mainnet-beta.solana.com"

# Database URL (optional, defaults to local SQLite)
export TURSO_URL="ore_rounds.db"  # Local file
```

## 📋 Usage

### API Server Mode (Recommended)

Start the REST API server:

```bash
# Start API server on port 8080
cargo run --release -- --mode api --port 8080

# Run in background
cargo run --release -- --mode api --port 8080 &
```

#### API Endpoints

Once the server is running:

```bash
# Health check
curl http://localhost:8080/
# Response: {"status": "healthy", "service": "ore-ingest"}

# Trigger background ingestion
curl http://localhost:8080/ingest
# Response: {"status": "started", "message": "Ingestion started in background", ...}

# List rounds with pagination
curl "http://localhost:8080/list?limit=5"
# Response: {"rounds": [...], "total": 1069, "limit": 5}
```

### CLI Ingestion Mode

Run ingestion directly:

```bash
# One-time ingestion (traditional mode)
cargo run --release -- --mode ingest

# Or without API features (minimal build)
cargo run --release --no-default-features
```

### CLI Options

```bash
# Show help
cargo run --release -- --help

# API mode with custom port
cargo run --release -- --mode api --port 3000

# Direct ingestion mode
cargo run --release -- --mode ingest
```

## 📊 API Reference

### Endpoints

#### `GET /`
Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "service": "ore-ingest"
}
```

#### `GET /ingest`
Triggers round data ingestion in the background.

**Response:**
```json
{
  "status": "started",
  "message": "Ingestion started in background",
  "database": "ore_rounds.db",
  "rpc": "https://api.mainnet-beta.solana.com"
}
```

#### `GET /list?limit=N`
Retrieves list of processed rounds with pagination.

**Query Parameters:**
- `limit` (optional): Number of rounds to return (default: 100)

**Response:**
```json
{
  "rounds": [
    {
      "id": 48676,
      "address": "92GoDnzBwriaxRyH1jL2FEAikiVfZf595B3HgMWnUc46",
      "winning_square": 18,
      "winning_row": 4,
      "winning_col": 4,
      "top_miner": "E4mbL9r6mSWQyHsxZ5pqLXiHBDegFmBmkun8zQZSx5A5",
      "top_miner_reward": 100000000000,
      "split_reward": false,
      "motherlode_hit": false,
      "motherlode_amount": 0,
      "total_deployed": 38361252634,
      "total_vaulted": 3645471864,
      "total_winnings": 32809246780,
      "winners_count": 895,
      "expires_at": 379362429,
      "created_at": "2025-11-10T10:51:46.425630Z"
    }
  ],
  "total": 1069,
  "limit": 5
}
```

## 🗄️ Database Schema

The database schema is defined in [`sql/schema.sql`](sql/schema.sql) and includes:

- **round_winners table** with all round data
- **Indexes** for efficient queries by ID, winning square, miner, and timestamps
- **Automatic schema initialization** when service starts

Key fields:
- `id` - Round identifier (primary key)
- `winning_square`, `winning_row`, `winning_col` - Grid coordinates
- `top_miner`, `top_miner_reward` - Winner information
- `total_deployed`, `total_vaulted`, `total_winnings` - Round statistics
- `expires_at` - Claim expiration timestamp

## 🏗️ Architecture

### Module Structure

```
ingest/
├── src/
│   ├── main.rs        # CLI entry point and mode selection
│   ├── api.rs         # HTTP API endpoints (feature-gated)
│   ├── blockchain.rs  # Solana RPC client
│   ├── database.rs    # Database operations
│   ├── ingest.rs      # Ingestion orchestrator
│   └── types.rs        # Data structures
├── Cargo.toml
└── README.md
```

### Flow Diagram

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   CLI/API   │    │   Database  │    │  Blockchain │
│   Entry     │───▶│ Operations  │◀───│    Client   │
└─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │
       │                   ▼                   │
       │            ┌─────────────┐            │
       └───────────▶│   Storage   │◀───────────┘
                    └─────────────┘
```

## ⚙️ Configuration

### Features

```toml
[features]
default = ["api"]    # API enabled by default
api = ["dep:axum", "dep:clap", "dep:tower-http"]
```

### Dependencies

- **Core**: `ore-api`, `solana-client`, `turso`, `tokio`
- **API**: `axum` (HTTP server), `clap` (CLI parsing), `tower-http` (CORS)
- **Data**: `serde`, `chrono`, `anyhow`

## 🛠️ Development

### Building

```bash
# Standard build (with API)
cargo build

# Minimal build (no API)
cargo build --no-default-features

# Release build
cargo build --release
```

### Development Mode

```bash
# Auto-reload during development
cargo install cargo-watch
cargo watch -x 'run -- --mode api --port 8080'
```

### Testing

```bash
# Test with local database
export TURSO_URL="test.db"
cargo test

# Test API endpoints
cargo run -- --mode api --port 8080 &
curl http://localhost:8080/health
```

## 🚨 Troubleshooting

### Common Issues

1. **Port already in use**
   ```bash
   # Kill existing process
   lsof -ti:8080 | xargs kill -9
   # Or use different port
   cargo run --features api -- --mode api --port 3000
   ```

2. **Database connection failed**
   ```bash
   # Check environment variables
   echo $TURSO_URL
   # Use local database
   export TURSO_URL="local.db"
   ```

3. **RPC rate limiting**
   ```bash
   # Use paid RPC endpoint
   export SOLANA_RPC="https://your-rpc-provider.com"
   ```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- --mode api

# Check background tasks
ps aux | grep ore-ingest
```
