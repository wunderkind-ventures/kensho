#!/bin/bash

# Load sample anime data into the backend
# This script ingests the anime-offline-database.json file

echo "🎌 Kenshō Sample Data Loader"
echo "============================"

# Check if data file exists
DATA_FILE="data/anime-offline-database.json"
if [ ! -f "$DATA_FILE" ]; then
    echo "❌ Data file not found: $DATA_FILE"
    echo "Please ensure you're running from the project root"
    exit 1
fi

# Check if jq is installed for JSON parsing
if ! command -v jq &> /dev/null; then
    echo "⚠️  jq is not installed. Installing it would help parse the data."
    echo "   Run: brew install jq (macOS) or apt-get install jq (Linux)"
fi

# Check if backend is running
echo "🔍 Checking backend status..."
HEALTH_CHECK=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/api/health)

if [ "$HEALTH_CHECK" != "200" ]; then
    echo "⚠️  Backend is not running or not healthy (status: $HEALTH_CHECK)"
    echo "   Please start the backend first:"
    echo "   cd backend && cargo run --bin backend-server"
    exit 1
fi

echo "✅ Backend is running"

# Count anime entries
if command -v jq &> /dev/null; then
    ANIME_COUNT=$(jq '.data | length' < "$DATA_FILE")
    echo "📊 Found $ANIME_COUNT anime entries to load"
    
    # Sample the first 100 anime for POC
    echo "📥 Loading first 100 anime entries..."
    
    # Extract and load data (simplified version)
    jq '.data[:100]' < "$DATA_FILE" > /tmp/sample-anime.json
    
    echo "📤 Sending data to backend..."
    # Note: This would need an actual ingestion endpoint
    # For now, we'll show what would be loaded
    
    echo "Sample of data to load:"
    jq '.[:3] | .[] | {title: .title, type: .type, episodes: .episodes, status: .status}' < /tmp/sample-anime.json
    
else
    echo "📊 Data file found at: $DATA_FILE (84MB)"
    echo "   Install jq to parse and load the data"
fi

echo ""
echo "📝 Note: To properly ingest data, you need to:"
echo "1. Ensure Redis is running: docker-compose up -d redis"
echo "2. Run the backend: cd backend && cargo run --bin backend-server"
echo "3. Use the ingestion CLI: cd backend && cargo run --bin ingest-anime"

echo ""
echo "🌐 Frontend is available at: http://localhost:8080"
echo "🚀 Backend API is at: http://localhost:3000/api"
echo ""
echo "Quick test endpoints:"
echo "  • Health: curl http://localhost:3000/api/health"
echo "  • Search: curl 'http://localhost:3000/api/search?q=spy'"
echo "  • Browse: curl http://localhost:3000/api/browse/season/2024/spring"