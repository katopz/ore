# ORE Docker Debugging Plan

## Problem Analysis
- ✅ Fixed static linking issue (444KB → 1.4MB binary)
- ❌ Application still exits immediately in Docker (works locally)
- 🎯 Need to isolate which component is causing failure

## Systematic Debugging Approach

### Phase 1: Axum Only (Minimal API Server)
**Goal:** Verify basic web framework works in Docker
**Method:** Comment out all non-axum dependencies
**Expected:** Simple web server that responds on port 3000

### Phase 2: Axum + Turso  
**Goal:** Test database layer
**Method:** Add Turso/SQLite dependencies only
**Expected:** Database initialization and basic API endpoints

### Phase 3: Axum + Solana
**Goal:** Test Solana SDK integration
**Method:** Add Solana dependencies only
**Expected:** Solana client initialization

### Phase 4: Full Stack (Axum + Turso + Solana)
**Goal:** Complete application
**Method:** All dependencies enabled
**Expected:** Full ore-ingest functionality

## Implementation Strategy

For each phase:
1. Create minimal main.rs with only required imports
2. Build and test in Docker
3. Check logs and container status
4. Only proceed if phase works
5. Add next component and repeat

## Debugging Commands
```bash
# Test each version
docker build -t ore-debug-phaseX .
docker run -d -p 3000:3000 --name ore-test-X -e PORT=3000 -e TURSO_URL=test.db ore-debug-phaseX
docker ps && docker logs ore-test-X
curl -s http://localhost:3000/ || echo "No response"
```

## Success Criteria
- Container stays running (docker ps shows it)
- Responds to HTTP requests (curl gets JSON)
- No immediate exit (exit code 0 without running)

## Phase 1 Results ✅
**Status:** SUCCESS
- ✅ Axum-only works perfectly in Docker
- ✅ Container stays running (`docker ps` shows it)
- ✅ Responds to HTTP requests (`curl` gets JSON: `{"status":"healthy","phase":"1-axum-only","service":"ore-ingest"}`)
- ✅ Binary size: 1.4MB (statically linked)
- ✅ No immediate exit issue

**Confirms:** Issue is NOT with:
- Docker static linking ✅
- Axum web framework ✅ 
- Basic container environment ✅

**Issue caused by:** One of the dependencies we removed:
- ❌ Turso/SQLite database integration
- ❌ Solana SDK integration  
- ❌ ore-api program integration

## Phase 2 Results ✅
**Status:** SUCCESS
- ✅ Axum + Turso works perfectly in Docker
- ✅ Container stays running (`docker ps` shows it)
- ✅ Database connection established successfully
- ✅ Database table created successfully
- ✅ Responds to HTTP requests (`curl` gets JSON)
- ✅ Binary size: ~2MB (with Turso dependencies)
- ✅ No immediate exit issue

**Working Command:**
```bash
docker build -t ore-debug-phase2 .
docker run -d -p 3000:3000 -e PORT=3000 -e TURSO_URL=test.db ore-debug-phase2
curl http://localhost:3000/  # ✅ {"status":"healthy","service":"ore-ingest","phase":"2-axum-turso"}
```

## Phase 3 Results ✅
**Status:** SUCCESS
- ✅ Axum + Turso + Solana works perfectly in Docker
- ✅ Container stays running (`docker ps` shows it)
- ✅ Database connection established successfully
- ✅ Solana client connects to devnet and returns blockhash
- ✅ All HTTP endpoints respond correctly including /solana
- ✅ Binary size: ~15MB (with all Solana dependencies)
- ✅ No immediate exit issue

**Working Command:**
```bash
docker build -t ore-debug-phase3 .
docker run -d -p 3000:3000 -e PORT=3000 -e TURSO_URL=test.db ore-debug-phase3
curl http://localhost:3000/  # ✅ {"status":"healthy","service":"ore-ingest","phase":"3-axum-turso-solana"}
curl http://localhost:3000/solana  # ✅ {"solana_status":"connected","latest_blockhash":"..."}
```

**BREAKTHROUGH DISCOVERY:** After systematic testing through 3 phases:
- ✅ Docker static linking - WORKING
- ✅ Axum web framework - WORKING
- ✅ Turso/SQLite database integration - WORKING  
- ✅ Solana SDK integration - WORKING
- ✅ Basic container environment - WORKING

**Issue MUST be caused by the only remaining dependency:**
- ❌ ore-api program integration - THIS IS THE CULPRIT

## Phase 3: Axum + Solana  
**Goal:** Test Solana SDK integration
**Method:** Add Solana dependencies back (excluding ore-api)
**Expected:** Solana client initialization

## Phase 4: Full Stack (Axum + Turso + Solana)
**Goal:** Complete application
**Method:** All dependencies enabled including ore-api
**Expected:** Full ore-ingest functionality

## Phase 4: Full Stack (Axum + Turso + Solana + ore-api)
**Goal:** Complete application with ore-api integration
**Method:** Add ore-api dependency back (the suspected culprit)
**Expected:** Should identify and fix the Docker exit issue

**Priority:** HIGH - This is where the actual Docker exit issue should manifest

## Next Steps
1. ✅ Phase 1 (Axum only) - COMPLETED SUCCESSFULLY
2. ✅ Phase 2 (Axum + Turso) - COMPLETED SUCCESSFULLY  
3. ✅ Phase 3 (Axum + Turso + Solana) - COMPLETED SUCCESSFULLY
4. 🎯 Implement Phase 4 (Full stack with ore-api) - CURRENT PHASE
5. Identify and fix the ore-api integration issue
6. Deploy working full application

**Systematic Debugging Summary:**
- Successfully eliminated 4 potential causes of Docker exit
- Isolated issue to ore-api program integration specifically  
- Ready to focus fix efforts on the actual problematic component
