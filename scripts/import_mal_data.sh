#!/bin/bash

# MAL Data Import Script
# ======================
# 
# This script provides convenient wrappers for importing MAL anime data
# into SurrealDB with various configurations.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_SCRIPT="$SCRIPT_DIR/import_mal_data.py"
DATA_DIR="$SCRIPT_DIR/../data"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
DB_URL="http://localhost:8000"
USERNAME="root"
PASSWORD="root"
LIMIT=""
SKIP_RELATIONSHIPS=""
SKIP_EPISODES=""

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Function to check if SurrealDB is running
check_surrealdb() {
    print_status "Checking SurrealDB connection..."
    if curl -s "$DB_URL/health" >/dev/null 2>&1; then
        print_success "SurrealDB is running at $DB_URL"
    else
        print_error "Cannot connect to SurrealDB at $DB_URL"
        print_status "Make sure SurrealDB is running. You can start it with:"
        echo "  docker-compose up -d surrealdb"
        exit 1
    fi
}

# Function to check if data files exist
check_data_files() {
    print_status "Checking data files..."
    
    if [[ ! -f "$DATA_DIR/anime-offline-database.json" ]]; then
        print_error "anime-offline-database.json not found in $DATA_DIR"
        exit 1
    fi
    
    if [[ ! -f "$DATA_DIR/myanimelist.json" ]]; then
        print_error "myanimelist.json not found in $DATA_DIR"
        exit 1
    fi
    
    print_success "Data files found"
}

# Function to check Python dependencies
check_python_deps() {
    print_status "Checking Python dependencies..."
    
    if ! command -v python3 &> /dev/null; then
        print_error "Python 3 is required but not installed"
        exit 1
    fi
    
    # Check if requests library is available
    if ! python3 -c "import requests" 2>/dev/null; then
        print_warning "requests library not found. Installing..."
        pip3 install requests
    fi
    
    print_success "Python dependencies OK"
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS] COMMAND

Commands:
  test       - Import a small sample (100 entries) for testing
  small      - Import 1000 entries with relationships
  medium     - Import 5000 entries with relationships  
  large      - Import 10000 entries with relationships
  full       - Import all entries (may take hours)
  anime-only - Import anime without relationships or episodes
  help       - Show this help message

Options:
  --db-url URL       SurrealDB URL (default: http://localhost:8000)
  --username USER    SurrealDB username (default: root)
  --password PASS    SurrealDB password (default: root)
  --limit N          Limit number of entries to import
  --skip-rels        Skip creating relationships
  --skip-episodes    Skip creating episode records
  --data-dir DIR     Directory containing data files (default: ../data)

Examples:
  $0 test                    # Import 100 entries for testing
  $0 small                   # Import 1000 entries
  $0 --limit 500 anime-only  # Import 500 anime without relationships
  $0 --db-url http://localhost:8001 medium  # Use custom DB URL

EOF
}

# Function to run the import
run_import() {
    local cmd_args=()
    
    # Add basic arguments
    cmd_args+=("--db-url" "$DB_URL")
    cmd_args+=("--username" "$USERNAME") 
    cmd_args+=("--password" "$PASSWORD")
    cmd_args+=("--data-dir" "$DATA_DIR")
    
    # Add optional arguments
    [[ -n "$LIMIT" ]] && cmd_args+=("--limit" "$LIMIT")
    [[ -n "$SKIP_RELATIONSHIPS" ]] && cmd_args+=("--skip-relationships")
    [[ -n "$SKIP_EPISODES" ]] && cmd_args+=("--skip-episodes")
    
    print_status "Running import with: python3 $PYTHON_SCRIPT ${cmd_args[*]}"
    python3 "$PYTHON_SCRIPT" "${cmd_args[@]}"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --db-url)
            DB_URL="$2"
            shift 2
            ;;
        --username)
            USERNAME="$2"
            shift 2
            ;;
        --password)
            PASSWORD="$2"
            shift 2
            ;;
        --limit)
            LIMIT="$2"
            shift 2
            ;;
        --skip-rels)
            SKIP_RELATIONSHIPS="true"
            shift
            ;;
        --skip-episodes)
            SKIP_EPISODES="true"
            shift
            ;;
        --data-dir)
            DATA_DIR="$2"
            shift 2
            ;;
        help)
            show_usage
            exit 0
            ;;
        test)
            COMMAND="test"
            shift
            ;;
        small)
            COMMAND="small"
            shift
            ;;
        medium)
            COMMAND="medium"
            shift
            ;;
        large)
            COMMAND="large"
            shift
            ;;
        full)
            COMMAND="full"
            shift
            ;;
        anime-only)
            COMMAND="anime-only"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Set command-specific parameters
case "$COMMAND" in
    test)
        LIMIT="100"
        print_status "Test mode: importing $LIMIT entries"
        ;;
    small)
        LIMIT="1000" 
        print_status "Small import: $LIMIT entries with relationships"
        ;;
    medium)
        LIMIT="5000"
        print_status "Medium import: $LIMIT entries with relationships"
        ;;
    large)
        LIMIT="10000"
        print_status "Large import: $LIMIT entries with relationships"
        ;;
    full)
        print_status "Full import: all entries (this may take hours)"
        print_warning "This will import all ~39K entries. Continue? (y/N)"
        read -r response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            print_status "Import cancelled"
            exit 0
        fi
        ;;
    anime-only)
        SKIP_RELATIONSHIPS="true"
        SKIP_EPISODES="true"
        [[ -z "$LIMIT" ]] && LIMIT="1000"
        print_status "Anime-only mode: importing $LIMIT anime without relationships or episodes"
        ;;
    *)
        print_error "No command specified"
        show_usage
        exit 1
        ;;
esac

# Pre-flight checks
check_surrealdb
check_data_files
check_python_deps

# Run the import
print_status "Starting MAL data import..."
start_time=$(date +%s)

if run_import; then
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    print_success "Import completed in ${duration}s"
else
    print_error "Import failed"
    exit 1
fi