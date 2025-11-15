# ORE Labs

Progressive learning labs demonstrating different architecture phases of the ORE project. Each lab builds upon the previous one, introducing new concepts and technologies.

## 🎯 Learning Path

### Phase 1: Basic Axum Server
**Location**: `phase1_axum_only/`

**Objective**: Learn the fundamentals of web server development with Axum

**Features**:
- Minimal web server with basic routing
- Health check endpoint
- JSON response handling
- Basic async/await patterns

**Prerequisites**: Basic Rust knowledge

**Run**:
```bash
cd phase1_axum_only
cargo run
```

**Test**:
```bash
curl http://localhost:3000/
```

---

### Phase 2: Axum + Turso Database  
**Location**: `phase2_axum_turso/`

**Objective**: Add database persistence to web applications

**Features**:
- Real database integration with Turso (SQLite)
- Database table creation and initialization
- CRUD operations (Create, Read)
- Proper error handling
- Production-ready Docker build with cargo-chef

**Prerequisites**: Phase 1 completion, basic database concepts

**Run**:
```bash
cd phase2_axum_turso
./simple_test.sh  # Automated testing
# or
cargo run        # Manual testing
```

**Endpoints**:
- `GET /` - Health check
- `GET /status` - Database status
- `POST /db/create` - Create new record
- `GET /db/list` - List all records

**Learnings**:
- Database connection management
- SQL operations with Rust
- Error handling in database operations
- Environment configuration

---

### Phase 3: Axum + Turso + Solana
**Location**: `phase3_axum_turso_solana/`

**Objective**: Integrate blockchain functionality with traditional web services

**Features**:
- All Phase 2 database functionality
- Solana blockchain integration
- Wallet balance queries
- Devnet/mainnet RPC connections
- Hybrid web2 + web3 architecture

**Prerequisites**: Phase 2 completion, basic blockchain concepts

**Run**:
```bash
cd phase3_axum_turso_solana
./simple_test.sh
```

**New Endpoints**:
- `GET /solana/info` - Solana network info
- `GET /solana/balance/{pubkey}` - Wallet balance

**Learnings**:
- Web3 integration patterns
- RPC client usage
- Error handling for blockchain operations
- API design for hybrid applications

---

### Phase 4: Full ORE Integration
**Location**: `phase4_full_ore/`

**Objective**: Production-ready ORE data ingestion service

**Features**:
- Complete API server for ORE round data
- CLI interface for data ingestion
- Production Docker configuration
- Database optimization
- Comprehensive error handling

**Prerequisites**: Phase 3 completion, understanding of ORE protocol

**Run**:
```bash
cd phase4_full_ore
# API Mode (default)
cargo run

# CLI Mode
cargo run --no-default-features --features cli

# Docker
./simple_test.sh
```

**Learnings**:
- Production architecture patterns
- Feature-based compilation
- Multiple service modes
- Real-world data ingestion
- Performance optimization

## 🛠️ Development Workflow

### Testing Labs

Each lab includes automated test scripts:

```bash
cd <lab_directory>
./simple_test.sh
```

This script will:
1. Build the Docker image
2. Run the service container
3. Test all endpoints
4. Display logs
5. Clean up containers

### Local Development

For iterative development:

```bash
cd <lab_directory>
cargo run                    # Local development
cargo build --release        # Production build
cargo check                  # Quick syntax check
```

### Docker Development

```bash
cd <lab_directory>
docker build -t <lab-name> .
docker run -p 3000:3000 <lab-name>
```

## 🏗️ Architecture Evolution

### Phase 1 → Phase 2
- **Add**: Database layer
- **Pattern**: Service + Repository
- **Storage**: In-memory → Persistent

### Phase 2 → Phase 3  
- **Add**: Blockchain integration
- **Pattern**: Hybrid API (web2 + web3)
- **Complexity**: Single data source → Multiple data sources

### Phase 3 → Phase 4
- **Add**: Production features
- **Pattern**: Full-featured service
- **Scope**: Prototype → Production system

## 📚 Learning Outcomes

After completing all phases, you will understand:

1. **Web Development**: Axum framework, HTTP APIs, async programming
2. **Database Design**: SQL, SQLite/Turso, connection management
3. **Blockchain Integration**: Solana RPC, wallet operations, web3 patterns
4. **Production Deployment**: Docker, configuration, monitoring
5. **System Architecture**: Service design, data modeling, error handling
6. **Development Practices**: Testing, CI/CD, feature flags

## 🚀 Beyond the Labs

These labs serve as foundation for:
- Building production blockchain APIs
- Creating data ingestion services
- Developing hybrid web2/web3 applications
- Understanding scalable system architecture

## 🤝 Contributing

To add new labs or improve existing ones:

1. Follow the established pattern
2. Include comprehensive test scripts
3. Add proper documentation
4. Ensure Docker compatibility
5. Test on multiple platforms

## 🔧 Common Issues

### Build Problems
```bash
# Clear Rust cache
cargo clean

# Clear Docker cache
docker system prune -a

# Rebuild with no cache
docker build --no-cache -t test-image .
```

### Database Issues
```bash
# Check database permissions
ls -la *.db

# Test database manually
sqlite3 test.db ".tables"
```

### Port Conflicts
```bash
# Kill processes using port 3000
lsof -ti:3000 | xargs kill -9

# Or use different port
PORT=3001 cargo run
```

## 📖 Additional Resources

- [Axum Documentation](https://docs.rs/axum/)
- [Turso Documentation](https://docs.turso.tech/)
- [Solana Documentation](https://docs.solana.com/)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)