# ORE Ingest

ORE round winner data ingestion service with API server and CLI interface.

## 🚀 Features

- **API Server**: REST API for data ingestion and querying
- **CLI Interface**: Command-line tool for local testing and debugging
- **Docker Support**: Production-ready containerized deployment
- **Multi-architecture**: Works on both Intel and ARM Mac

## 🏗️ Architecture

### Service Modes

#### API Mode (Default)
Production-ready web API server with endpoints:
- `GET /` - Health check
- `GET /ingest` - Trigger data ingestion
- `GET /list?limit=N` - List rounds with pagination

#### CLI Mode
Command-line interface for local development:
- Direct data ingestion from Solana blockchain
- Database initialization and testing
- Debugging and troubleshooting

## 🐳 Docker Deployment

### Quick Start

```bash
# Build image (production optimized)
docker build -t ore-ingest:latest .

# Run API server (default mode)
docker run -d \
  -p 4000:4000 \
  --name ore-ingest \
  -e PORT=4000 \
  -e TURSO_URL=/app/data/ore.db \
  ore-ingest

# Test API
curl http://localhost:4000/
```

### Image Size Optimization

| Base Image | Size | Build Time | Compatibility | Status |
|------------|------|------------|-------------|---------|
| Ubuntu 22.04 | 127MB | ~3 min | ✅ Excellent | Recommended |
| Alpine Linux | 56.8MB | ~5 min | ⚠️ Linking Issues | Experimental |

**Recommendation**: Use Ubuntu 22.04-based Dockerfile for production. While Alpine is smaller, it has dynamic linking issues with cross-compilation on ARM Mac.

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `4000` | API server port |
| `TURSO_URL` | `/app/data/ore.db` | SQLite database path |
| `SOLANA_RPC` | `https://api.mainnet-beta.solana.com` | Solana RPC endpoint |

### Production Deployment

```bash
# Build with specific platform if needed
docker build --platform=linux/amd64 -t ore-ingest:prod .

# Run with database persistence
docker run -d \
  -p 4000:4000 \
  --name ore-ingest-prod \
  -v $(pwd)/data:/app/data \
  -e PORT=4000 \
  -e TURSO_URL=/app/data/ore.db \
  -e SOLANA_RPC=https://api.mainnet-beta.solana.com \
  ore-ingest:prod

# Check container health
docker ps --format "table {{.Names}}\t{{.Status}}"

# View logs
docker logs ore-ingest-prod
```

### Health Checks

The container includes built-in health monitoring:

```bash
# Check container health status
docker inspect --format='{{.State.Health.Status}}' ore-ingest-prod

# Manual health check
curl -f http://localhost:4000/ || exit 1
```

## 🔧 Local Development

### Prerequisites

- Rust 1.70+
- SQLite
- Docker (optional)

### Build & Run

```bash
# Build the project
cargo build --release

# Run API server (default)
cargo run

# Run CLI only
cargo run --no-default-features --features cli
```

### Development with Docker

```bash
# Build for development
docker build -t ore-ingest:dev .

# Mount source for hot reloading
docker run -d \
  -p 4000:4000 \
  -v $(pwd)/ingest:/app/ingest \
  -v $(pwd)/api:/app/api \
  --name ore-ingest-dev \
  -e PORT=4000 \
  ore-ingest:dev

# View logs
docker logs -f ore-ingest-dev
```

## 📚 API Reference

### Endpoints

#### Health Check
```http
GET /
```

**Response:**
```json
{
  "service": "ore-ingest",
  "status": "healthy"
}
```

#### Trigger Ingestion
```http
GET /ingest
```

**Response:**
```json
{
  "message": "Ingestion started in background",
  "status": "started",
  "database": "/app/data/ore.db",
  "rpc": "https://api.mainnet-beta.solana.com"
}
```

#### List Rounds
```http
GET /list?limit=10
```

**Response:**
```json
{
  "rounds": [
    {
      "id": 53125,
      "address": "...",
      "winning_square": 42,
      "winning_row": 4,
      "winning_col": 2,
      "top_miner": "...",
      "top_miner_reward": 1000000,
      "split_reward": false,
      "motherlode_hit": false,
      "motherlode_amount": 0,
      "total_deployed": 1000000000,
      "total_vaulted": 500000000,
      "total_winnings": 750000000,
      "winners_count": 100,
      "expires_at": 1704067200,
      "created_at": "2023-12-31T23:59:59Z"
    }
  ],
  "total": 53125,
  "limit": 10
}
```

## 🔍 CLI Usage

### Basic Ingestion

```bash
# Run full data ingestion
cargo run --no-default-features --features cli

# With custom database
TURSO_URL=/path/to/custom.db cargo run --no-default-features --features cli

# With custom RPC endpoint
SOLANA_RPC=https://api.devnet.solana.com cargo run --no-default-features --features cli
```

### Environment Variables

Same environment variables as API mode are supported.

## 🏗️ Build Configuration

### Features

The project uses Rust features for conditional compilation:

```toml
[features]
default = ["api"]  # API server by default
api = ["dep:axum", "dep:tower-http"]  # Web API dependencies
cli = ["dep:clap"]                     # CLI dependencies
```

### Build Variants

```bash
# API server (default)
cargo build --features api

# CLI only
cargo build --features cli

# Must specify exactly one feature
cargo build --no-default-features --features cli  # ✅ Works
cargo build --no-default-features --features api   # ✅ Works
cargo build --no-default-features                # ❌ Error: Must specify feature
cargo build --features "api,cli"                 # ❌ Error: Cannot enable both
```

## 🐛 Troubleshooting

### Docker Issues

#### ARM Mac Build Errors

If you encounter ARM compilation errors:

```bash
# Clear Docker cache and rebuild
docker system prune -a
docker build --no-cache -t ore-ingest .

# Use specific platform if needed
docker build --platform=linux/amd64 -t ore-ingest .
```

#### Container Exits Immediately

1. **Check logs**: `docker logs <container-name>`
2. **Verify environment variables**: Ensure required env vars are set
3. **Check port conflicts**: Ensure port 4000 is available

#### Database Issues

```bash
# Check database directory permissions
docker exec ore-ingest ls -la /app/data

# Test database connectivity
docker exec ore-ingest sqlite3 /app/data/ore.db ".tables"
```

### Local Development Issues

#### Build Errors

```bash
# Update dependencies
cargo update

# Clean build
cargo clean
cargo build --release

# Check toolchain
rustup update
rustc --version
```

### Docker Image Comparison

| Image | Size | Status | Use Case |
|-------|------|--------|-----------|
| `ore-ingest:latest` | 127MB | ✅ Production ready |
| `ore-ingest:alpine` | 56.8MB | ⚠️ Development only |
| `ore-ingest:slim` | 127MB | ✅ Alternative production |

**Note**: Alpine builds achieve 55% size reduction (56.8MB vs 127MB) but have cross-compilation issues on ARM Mac. Ubuntu 22.04 builds with optimizations provide best balance of size and reliability.

#### Runtime Errors

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Test database
sqlite3 ore.db ".tables"
```

## 📊 Monitoring & Observability

### Health Monitoring

```bash
# Container health status
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

# Detailed health info
docker inspect --format='{{json .State.Health}}' ore-ingest

# API health check
curl -s http://localhost:4000/ | jq .
```

### Log Analysis

```bash
# Follow logs in real-time
docker logs -f ore-ingest

# Filter logs for errors
docker logs ore-ingest 2>&1 | grep -i error

# Log volume metrics
docker stats ore-ingest --no-stream
```

## 🔧 Configuration

### Production Best Practices

1. **Database Persistence**: Mount volumes for data persistence
2. **Resource Limits**: Set memory and CPU limits
3. **Health Checks**: Use built-in health monitoring
4. **Security**: Run as non-root user (default in Dockerfile)
5. **Networking**: Use proper network isolation

### Example Production Config

```yaml
# docker-compose.yml
version: '3.8'
services:
  ore-ingest:
    build: .
    ports:
      - "4000:4000"
    environment:
      - PORT=4000
      - TURSO_URL=/app/data/ore.db
      - SOLANA_RPC=https://api.mainnet-beta.solana.com
    volumes:
      - ./data:/app/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:4000/"]
      interval: 30s
      timeout: 10s
      retries: 3
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '0.5'
```

## 📈 Performance

### Optimization Tips

1. **Database**: Use SQLite with proper indexing
2. **Network**: Choose nearest Solana RPC endpoint
3. **Container**: Use resource limits for stability
4. **Caching**: Enable Redis caching for frequent queries

### Benchmarks

- **API Response Time**: < 100ms for health checks
- **Ingestion Speed**: ~100 rounds/second
- **Memory Usage**: ~50MB baseline
- **CPU Usage**: < 10% during normal operation

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

### Development Setup

```bash
git clone <your-fork>
cd ore
cargo build
cargo test
```

## 📄 License

Apache License 2.0 - see LICENSE file for details.