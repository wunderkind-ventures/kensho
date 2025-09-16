#!/bin/bash

echo "=== Testing Kenshō Infrastructure ==="
echo

# Test SurrealDB
echo "1. Testing SurrealDB..."
SURREAL_RESPONSE=$(curl -s -X POST http://localhost:8000/sql \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -H "Surreal-NS: kensho" \
  -H "Surreal-DB: poc" \
  -u "root:root" \
  -d '"INFO FOR DB;"' 2>&1)

if echo "$SURREAL_RESPONSE" | grep -q '"status":"OK"'; then
    echo "   ✅ SurrealDB is running and accessible"
else
    echo "   ❌ SurrealDB connection failed"
    echo "   Response: $SURREAL_RESPONSE"
fi

# Test Redis
echo
echo "2. Testing Redis..."
REDIS_RESPONSE=$(docker exec kensho-redis redis-cli -a kensho_redis_pass ping 2>/dev/null)

if [ "$REDIS_RESPONSE" = "PONG" ]; then
    echo "   ✅ Redis is running and accessible"
else
    echo "   ❌ Redis connection failed"
    echo "   Response: $REDIS_RESPONSE"
fi

# Test Backend Build
echo
echo "3. Testing Backend Build..."
if [ -f "backend/target/debug/kensho-backend" ]; then
    echo "   ✅ Backend binary exists"
else
    echo "   ⚠️  Backend not built yet (run: cd backend && cargo build)"
fi

# Test Frontend Build
echo
echo "4. Testing Frontend Build..."
if [ -f "frontend/target/wasm32-unknown-unknown/debug/kensho-frontend.wasm" ]; then
    echo "   ✅ Frontend WASM binary exists"
else
    echo "   ⚠️  Frontend not built yet (run: cd frontend && cargo build --target wasm32-unknown-unknown)"
fi

# Check Docker containers
echo
echo "5. Docker Container Status:"
docker ps --format "table {{.Names}}\t{{.Status}}" | grep kensho

echo
echo "=== Infrastructure Test Complete ==="