#!/bin/bash
set -e
echo "🧪 Phase 1: x86_64 Platform Test"
echo "Building with platform=linux/amd64..."
docker build --platform=linux/amd64 -t phase1-test . || { echo "Build failed"; exit 1; }
echo ""
echo "Running container..."
docker run -d --name phase1-test -p 3000:3000 phase1-test
echo "Waiting 3 seconds..."
sleep 3
echo "Testing endpoints:"
curl -s http://localhost:3000/ || echo "Health endpoint FAILED"
echo ""
curl -s http://localhost:3000/status || echo "Status endpoint FAILED"
echo ""
echo "Container logs:"
docker logs phase1-test
echo ""
echo "Cleaning up..."
docker rm -f phase1-test
echo "✅ Test complete"
