#!/bin/bash
set -e
echo "🧪 Phase 2: Axum + Turso Database Test"
echo "Building with platform=linux/amd64..."
docker build --platform=linux/amd64 -t phase2-test . || { echo "Build failed"; exit 1; }
echo ""
echo "Running container..."
docker run -d --name phase2-test -p 3000:3000 phase2-test
echo "Waiting 5 seconds..."
sleep 5
echo ""
echo "Testing endpoints:"
echo "=== Health Check ==="
curl -s http://localhost:3000/ || echo "Health endpoint FAILED"
echo ""
echo "=== Status Check ==="
curl -s http://localhost:3000/status || echo "Status endpoint FAILED"
echo ""
echo "=== Database: Create Record ==="
curl -s -X POST http://localhost:3000/db/create || echo "Create record FAILED"
echo ""
echo "=== Database: List Records ==="
curl -s http://localhost:3000/db/list || echo "List records FAILED"
echo ""
echo "=== Create Another Record ==="
curl -s -X POST http://localhost:3000/db/create || echo "Second create FAILED"
echo ""
echo "=== List Records Again ==="
curl -s http://localhost:3000/db/list || echo "Second list FAILED"
echo ""
echo "Container logs:"
docker logs phase2-test
echo ""
echo "Cleaning up..."
docker rm -f phase2-test
echo ""
echo "✅ Phase 2 Test Complete"
