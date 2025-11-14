#!/bin/bash
set -e

echo "🧪 Phase 1: Axum Only - Ubuntu Container Test"
echo "Goal: Build and run minimal axum server inside Ubuntu container"
echo ""

LOG_FILE="axum_only_test.log"
BUILD_LOG="build.log"
RUN_LOG="run.log"

echo "📝 Logging to: $LOG_FILE"
echo "🔨 Build log: $BUILD_LOG"
echo "🚀 Run log: $RUN_LOG"
echo ""

echo "🧹 Cleaning up previous containers..."
docker rm -f axum-only-test 2>/dev/null || true
docker rmi axum-only-lab 2>/dev/null || true

echo "📦 Building Ubuntu container with axum-only..."
docker build -f Dockerfile.ubuntu -t axum-only-lab . > $BUILD_LOG 2>&1 || {
    echo "❌ BUILD FAILED"
    echo "Check $BUILD_LOG for details"
    tail -20 $BUILD_LOG
    exit 1
}

echo "✅ Build successful"

echo ""
echo "🚀 Running container in background..."
docker run -d \
    --name axum-only-test \
    -p 3000:3000 \
    axum-only-lab > $RUN_LOG 2>&1 || {
    echo "❌ CONTAINER START FAILED"
    echo "Check $RUN_LOG for details"
    cat $RUN_LOG
    exit 1
}

echo "⏳ Waiting for container to start..."
sleep 5

if ! docker ps | grep axum-only-test; then
    echo "❌ Container not running!"
    echo "Container logs:"
    docker logs axum-only-test
    exit 1
fi

echo "✅ Container is running"

echo ""
echo "🔍 Testing API endpoints..."

echo "Testing /:"
curl -s http://localhost:3000/ || echo "❌ Health endpoint failed"

echo ""
echo "Testing /status:"
curl -s http://localhost:3000/status || echo "❌ Status endpoint failed"

echo ""
echo "Testing /test/world:"
curl -s http://localhost:3000/test/world || echo "❌ Test endpoint failed"

echo ""
echo "📊 Container logs:"
docker logs axum-only-test

echo ""
echo "🧹 Cleaning up..."
docker rm -f axum-only-test

echo ""
echo "✅ Phase 1 Test Complete"
echo "Result: Axum-only builds and runs in Ubuntu container with x86_64 cross-compilation"
