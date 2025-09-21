#!/bin/bash

echo "🚀 Starting Kenshō stack..."

# Build images if needed
echo "Building Docker images..."
docker-compose build

# Start services
echo "Starting services..."
docker-compose up -d

# Wait for services to be healthy
echo "Waiting for services to be ready..."
sleep 5

# Check service status
echo "Checking service status..."
docker-compose ps

echo ""
echo "✅ Services are running:"
echo "  - Frontend: http://localhost:8080"
echo "  - Backend API: http://localhost:3000"
echo "  - SurrealDB: http://localhost:8000"
echo "  - Redis: localhost:6379"
echo ""
echo "To view logs: docker-compose logs -f"
echo "To stop: docker-compose down"