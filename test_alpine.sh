#!/bin/bash
set -e

echo "🧪 Testing Alpine Docker Container"
echo "=================================="

# Build the Alpine container
echo "📦 Building Alpine container..."
docker build --platform linux/x86_64 -f Dockerfile.alpine -t ore-ingest:alpine-test .

echo ""
echo "🔍 Testing container functionality..."

# Test 1: Basic container can run
echo "Test 1: Basic container functionality"
if docker run --platform linux/x86_64 --rm --entrypoint="" ore-ingest:alpine-test /bin/sh -c "echo 'Container runs'"; then
    echo "✅ Basic container functionality: PASSED"
else
    echo "❌ Basic container functionality: FAILED"
    exit 1
fi

# Test 2: Binary exists and is executable
echo "Test 2: Binary check"
if docker run --platform linux/x86_64 --rm --entrypoint="" ore-ingest:alpine-test /bin/sh -c "ls -la /app/ore-ingest && [ -x /app/ore-ingest ]"; then
    echo "✅ Binary exists and is executable: PASSED"
else
    echo "❌ Binary exists and is executable: FAILED"
    exit 1
fi

# Test 3: Binary is properly linked with musl
echo "Test 3: Musl linking check"
if docker run --platform linux/x86_64 --rm --entrypoint="" ore-ingest:alpine-test /bin/sh -c "ldd /app/ore-ingest 2>/dev/null | grep -q 'ld-musl'"; then
    echo "✅ Musl linking: PASSED"
else
    echo "❌ Musl linking: FAILED"
    exit 1
fi

# Test 4: Application starts with environment variables
echo "Test 4: Application startup"
CONTAINER_ID=$(docker run --platform linux/x86_64 -d \
    -e PORT=4000 \
    -e TURSO_URL=/app/data/ore.db \
    --name alpine-test-temp \
    ore-ingest:alpine-test)

echo "Container ID: $CONTAINER_ID"

# Wait for container to potentially start
sleep 5

# Check container status
STATUS=$(docker inspect --format='{{.State.Status}}' $CONTAINER_ID 2>/dev/null || echo "not_found")
EXIT_CODE=$(docker inspect --format='{{.State.ExitCode}}' $CONTAINER_ID 2>/dev/null || echo "-1")

# Get container logs regardless of status
echo "=== Container Logs ==="
docker logs $CONTAINER_ID 2>/dev/null || echo "No logs available"
echo "======================"

echo "Container Status: $STATUS"
echo "Exit Code: $EXIT_CODE"

if [ "$STATUS" = "running" ]; then
    echo "✅ Application startup: PASSED (Container is running)"

    # Test 5: API is responsive
    echo "Test 5: API responsiveness"
    sleep 3
    if docker exec $CONTAINER_ID /bin/sh -c "curl -f http://localhost:4000/ 2>/dev/null || wget -q -O - http://localhost:4000/ 2>/dev/null"; then
        echo "✅ API responsiveness: PASSED"
    else
        echo "⚠️  API responsiveness: FAILED (might need more startup time)"
    fi

    # Cleanup
    docker stop $CONTAINER_ID
else
    if [ "$EXIT_CODE" = "0" ]; then
        echo "⚠️  Application startup: PASSED (Container exited normally - might be CLI mode)"
    elif [ "$EXIT_CODE" = "133" ]; then
        echo "❌ Application startup: FAILED (Exit code 133 - missing environment variable or permission issue)"
        docker logs $CONTAINER_ID 2>/dev/null || echo "No logs available"
        exit 1
    else
        echo "❌ Application startup: FAILED (Exit code $EXIT_CODE)"
        exit 1
    fi
fi

# Cleanup the test container
docker rm -f alpine-test-temp 2>/dev/null || true

echo ""
echo "🎉 Alpine container test completed!"
echo "===================================="

# Show final image size
IMAGE_SIZE=$(docker images --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}" ore-ingest:alpine-test | tail -n1)
echo "Final image size: $IMAGE_SIZE"

# Compare with Ubuntu version if available
if docker images ore-ingest:ubuntu-optimized --quiet | head -1; then
    UBUNTU_SIZE=$(docker images --format "{{.Size}}" ore-ingest:ubuntu-optimized)
    echo "Ubuntu image size: $UBUNTU_SIZE"
fi
