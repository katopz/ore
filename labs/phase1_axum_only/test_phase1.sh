#!/bin/bash
set -e

echo "🧪 Phase 1: Axum Only - Ubuntu Container Cross-Compilation Test"
echo "Goal: Build x86_64 binary inside Ubuntu container, run in production-like environment"
echo ""

LOG_DIR="logs"
mkdir -p $LOG_DIR

BUILD_LOG="$LOG_DIR/phase1_build.log"
RUN_LOG="$LOG_DIR/phase1_run.log"
TEST_LOG="$LOG_DIR/phase1_test.log"

echo "📝 Build log: $BUILD_LOG"
echo "🚀 Run log: $RUN_LOG"
echo "🧪 Test log: $TEST_LOG"
echo ""

# Clean up
echo "🧹 Cleaning up previous containers..."
docker rm -f phase1-test 2>/dev/null || true
docker rmi phase1-lab 2>/dev/null || true

echo "📦 Building inside Ubuntu container (ARM Mac → x86_64 cross-compilation)..."
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock -v "$(pwd)":/workspace ubuntu:20.04 sh -c "
apt-get update >/dev/null 2>&1 && 
apt-get install -y curl build-essential pkg-config >/dev/null 2>&1 && 
curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y >/dev/null 2>&1 && 
. /root/.cargo/env && 
rustup target add x86_64-unknown-linux-gnu >/dev/null 2>&1 && 
cd /workspace/labs/phase1_axum_only && 
export CARGO_BUILD_TARGET=x86_64-unknown-linux-gnu && 
cargo build --release --target x86_64-unknown-linux-gnu && 
echo === BUILD SUCCESS === && 
ls -la target/x86_64-unknown-linux-gnu/release/phase1-axum-only && 
file target/x86_64-unknown-linux-gnu/release/phase1-axum-only && 
echo === BINARY SIZE === && 
du -h target/x86_64-unknown-linux-gnu/release/phase1-axum-only
" > $BUILD_LOG 2>&1

if [ $? -eq 0 ]; then
    echo "✅ Build successful"
    echo "📊 Build output:"
    tail -10 $BUILD_LOG
else
    echo "❌ BUILD FAILED"
    echo "Check $BUILD_LOG for details"
    tail -20 $BUILD_LOG
    exit 1
fi

echo ""
echo "🏗️ Creating runtime container image..."
# Copy the binary out and create a runtime container
docker run --rm -v "$(pwd)":/workspace ubuntu:20.04 sh -c "
mkdir -p /tmp/phase1 && 
cp /workspace/labs/phase1_axum_only/target/x86_64-unknown-linux-gnu/release/phase1-axum-only /tmp/phase1/ && 
apt-get update >/dev/null 2>&1 && 
apt-get install -y curl >/dev/null 2>&1
" &&

docker create --name phase1-temp \
  ubuntu:20.04 &&

docker cp /workspace/labs/phase1_axum_only/target/x86_64-unknown-linux-gnu/release/phase1-axum-only phase1-temp:/app/ &&

docker commit phase1-temp phase1-lab &&

docker rm phase1-temp &&

echo "✅ Runtime image created"

echo ""
echo "🚀 Starting container in background..."
docker run -d \
    --name phase1-test \
    -p 3000:3000 \
    phase1-lab \
    /app/phase1-axum-only > $RUN_LOG 2>&1 || {
    echo "❌ CONTAINER START FAILED"
    cat $RUN_LOG
    exit 1
}

echo "⏳ Waiting for container to start..."
sleep 5

# Check if container is running
if ! docker ps | grep phase1-test; then
    echo "❌ Container not running!"
    echo "Container logs:"
    docker logs phase1-test
    exit 1
fi

echo "✅ Container is running"

echo ""
echo "🧪 Testing API endpoints..."
{
    echo "Testing /health endpoint:"
    curl -s http://localhost:3000/ || echo "❌ Health endpoint failed"
    
    echo ""
    echo "Testing /status endpoint:"
    curl -s http://localhost:3000/status || echo "❌ Status endpoint failed"
    
    echo ""
    echo "Checking container status:"
    docker ps | grep phase1-test
    
    echo ""
    echo "Container logs:"
    docker logs phase1-test
} > $TEST_LOG 2>&1

echo "📊 Test results:"
cat $TEST_LOG

echo ""
echo "🧹 Cleaning up..."
docker rm -f phase1-test

echo ""
echo "✅ Phase 1 Test Complete"
echo "Result: Axum-only cross-compiled from ARM to x86_64 and runs in Ubuntu container"
