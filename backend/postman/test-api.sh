#!/bin/bash

# Kenshō API Test Script
# Usage: ./test-api.sh [endpoint]

BASE_URL="${BASE_URL:-http://localhost:3000}"
AUTH_TOKEN=""
REFRESH_TOKEN=""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test data
TEST_EMAIL="${CRUNCHYROLL_EMAIL:-test@example.com}"
TEST_PASSWORD="${CRUNCHYROLL_PASSWORD:-password}"
TEST_ANIME_ID="550e8400-e29b-41d4-a716-446655440000"
TEST_EPISODE_NUM="1"
TEST_SEARCH_QUERY="spy family"
TEST_YEAR="2024"
TEST_SEASON="spring"

# Helper function to print colored output
print_test() {
    echo -e "${BLUE}Testing: $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Test health endpoint
test_health() {
    print_test "GET /api/health"
    response=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/health")
    http_code=$(echo "$response" | tail -n1)
    # macOS compatible way to get all but last line
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        print_success "Health check passed: $body"
    else
        print_error "Health check failed with status $http_code"
    fi
}

# Test login
test_login() {
    print_test "POST /api/auth/login"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASSWORD\"}")
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        AUTH_TOKEN=$(echo "$body" | grep -o '"token":"[^"]*' | sed 's/"token":"//')
        REFRESH_TOKEN=$(echo "$body" | grep -o '"refresh_token":"[^"]*' | sed 's/"refresh_token":"//')
        print_success "Login successful, token saved"
        echo "Token: ${AUTH_TOKEN:0:20}..."
    else
        print_error "Login failed with status $http_code"
        echo "$body" | jq '.' 2>/dev/null || echo "$body"
    fi
}

# Test search
test_search() {
    print_test "GET /api/search"
    # URL encode the search query
    encoded_query=$(echo -n "$TEST_SEARCH_QUERY" | sed 's/ /%20/g')
    response=$(curl -s -w "\n%{http_code}" "${BASE_URL}/api/search?q=${encoded_query}&limit=5")
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        print_success "Search successful"
        result_count=$(echo "$body" | jq '.results | length' 2>/dev/null || echo "0")
        echo "Found $result_count results"
    else
        print_error "Search failed with status $http_code"
        echo "$body"
    fi
}

# Test get anime
test_get_anime() {
    print_test "GET /api/anime/{id}"
    response=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/anime/$TEST_ANIME_ID")
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        print_success "Get anime successful"
        echo "$body" | jq '.anime.title' 2>/dev/null || echo "Anime found"
    elif [ "$http_code" = "404" ]; then
        print_error "Anime not found (expected if database is empty)"
    else
        print_error "Get anime failed with status $http_code"
    fi
}

# Test get episodes
test_get_episodes() {
    print_test "GET /api/anime/{id}/episodes"
    response=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/anime/$TEST_ANIME_ID/episodes")
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        print_success "Get episodes successful"
        echo "$body" | jq '.total' 2>/dev/null && echo "episodes found"
    elif [ "$http_code" = "404" ]; then
        print_error "Anime not found (expected if database is empty)"
    else
        print_error "Get episodes failed with status $http_code"
    fi
}

# Test browse season
test_browse_season() {
    print_test "GET /api/browse/season/{year}/{season}"
    response=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/browse/season/$TEST_YEAR/$TEST_SEASON")
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        print_success "Browse season successful"
        echo "$body" | jq '.total' 2>/dev/null && echo "anime found for $TEST_SEASON $TEST_YEAR"
    else
        print_error "Browse season failed with status $http_code"
    fi
}

# Test stream (requires auth)
test_stream() {
    if [ -z "$AUTH_TOKEN" ]; then
        print_error "No auth token available. Run login test first."
        return
    fi
    
    print_test "GET /api/stream/{anime_id}/{episode}"
    response=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/stream/$TEST_ANIME_ID/$TEST_EPISODE_NUM" \
        -H "Authorization: Bearer $AUTH_TOKEN")
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        print_success "Get stream successful"
        echo "$body" | jq '.streams[0].url' 2>/dev/null || echo "Stream URL retrieved"
    elif [ "$http_code" = "401" ]; then
        print_error "Authentication required or token expired"
    else
        print_error "Get stream failed with status $http_code"
    fi
}

# Test logout
test_logout() {
    if [ -z "$AUTH_TOKEN" ]; then
        print_error "No auth token available. Run login test first."
        return
    fi
    
    print_test "POST /api/auth/logout"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/auth/logout" \
        -H "Authorization: Bearer $AUTH_TOKEN")
    
    http_code=$(echo "$response" | tail -n1)
    
    if [ "$http_code" = "200" ]; then
        print_success "Logout successful"
        AUTH_TOKEN=""
    else
        print_error "Logout failed with status $http_code"
    fi
}

# Test refresh token
test_refresh() {
    if [ -z "$REFRESH_TOKEN" ]; then
        print_error "No refresh token available. Run login test first."
        return
    fi
    
    print_test "POST /api/auth/refresh"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/auth/refresh" \
        -H "Content-Type: application/json" \
        -d "{\"refresh_token\":\"$REFRESH_TOKEN\"}")
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        AUTH_TOKEN=$(echo "$body" | grep -o '"token":"[^"]*' | sed 's/"token":"//')
        print_success "Token refreshed successfully"
    else
        print_error "Token refresh failed with status $http_code"
    fi
}

# Run all tests
run_all_tests() {
    echo "=== Kenshō API Test Suite ==="
    echo "Base URL: $BASE_URL"
    echo ""
    
    test_health
    echo ""
    
    test_search
    echo ""
    
    test_get_anime
    echo ""
    
    test_get_episodes
    echo ""
    
    test_browse_season
    echo ""
    
    test_login
    echo ""
    
    if [ -n "$AUTH_TOKEN" ]; then
        test_stream
        echo ""
        
        test_refresh
        echo ""
        
        test_logout
        echo ""
    fi
    
    echo "=== Test Suite Complete ==="
}

# Main script logic
case "${1:-all}" in
    health)
        test_health
        ;;
    login)
        test_login
        ;;
    search)
        test_search
        ;;
    anime)
        test_get_anime
        ;;
    episodes)
        test_get_episodes
        ;;
    browse)
        test_browse_season
        ;;
    stream)
        test_stream
        ;;
    logout)
        test_logout
        ;;
    refresh)
        test_refresh
        ;;
    all)
        run_all_tests
        ;;
    *)
        echo "Usage: $0 [health|login|search|anime|episodes|browse|stream|logout|refresh|all]"
        echo ""
        echo "Environment variables:"
        echo "  BASE_URL - API base URL (default: http://localhost:3000)"
        echo "  CRUNCHYROLL_EMAIL - Your Crunchyroll email"
        echo "  CRUNCHYROLL_PASSWORD - Your Crunchyroll password"
        exit 1
        ;;
esac