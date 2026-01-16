#!/bin/bash

# ==============================================================================
# LeeAttend Test Environment "Abstract Parent"
# Handles: Postgres Setup, Isolated DBs, Migrations, and Cleanup.
# ==============================================================================

set -euo pipefail

# --- Configuration ---
export PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export COMPOSE_FILE="$PROJECT_ROOT/docker-compose.test.yml"
export LOG_DIR="$PROJECT_ROOT/.test_logs"
mkdir -p "$LOG_DIR"

# Common Variables
export DATABASE_URL="postgres://test_user:test_password@localhost:5433/test_database"
CPU_CORES=$(nproc)
export NUM_WORKERS=$(( CPU_CORES / 2 ))
[ $NUM_WORKERS -lt 1 ] && export NUM_WORKERS=1

# ANSI Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# --- Methods (The "Interface" Logic) ---

leeattend_cleanup() {
    echo -e "\n${YELLOW}🧹 [Parent] Cleaning up test environment...${NC}"
    docker-compose -f "$COMPOSE_FILE" down -v > /dev/null 2>&1
    rm -f apiserver/.api_port* webdashboard/.web_port*
    echo -e "${GREEN}✅ Cleanup complete.${NC}"
}

leeattend_setup() {
    echo -e "${BLUE}🏗️ [Parent] Setting up Postgres Environment...${NC}"
    
    docker-compose -f "$COMPOSE_FILE" up -d test_db > /dev/null 2>&1

    echo -n "  Waiting for database to be ready..."
    until docker-compose -f "$COMPOSE_FILE" exec test_db pg_isready -U test_user -d test_database > /dev/null 2>&1; do
        printf "."
        sleep 1
    done
    echo -e " ${GREEN}Ready!${NC}"

    echo -e "  Creating isolated databases for $NUM_WORKERS workers..."
    for i in $(seq 0 $((NUM_WORKERS-1))); do
        docker-compose -f "$COMPOSE_FILE" exec test_db psql -U test_user -d postgres -c "CREATE DATABASE test_database_e2e_$i" > /dev/null 2>&1 || true
    done

    echo -e "  Running migrations..."
    (cd apiserver && sqlx migrate run --database-url "$DATABASE_URL") > "$LOG_DIR/parent_mig_main.log" 2>&1
    for i in $(seq 0 $((NUM_WORKERS-1))); do
        (cd apiserver && sqlx migrate run --database-url "postgres://test_user:test_password@localhost:5433/test_database_e2e_$i") > "$LOG_DIR/parent_mig_e2e_$i.log" 2>&1
    done
    
    echo -e "${GREEN}✅ Infrastructure ready.${NC}"
}

# --- execution Logic ---

run_within_env() {
    local KEEP_ALIVE=false
    if [[ "${1:-}" == "--keep-alive" ]]; then
        KEEP_ALIVE=true
        shift
    fi

    # 1. Setup
    leeattend_setup

    # 2. Execute child logic if provided
    if [ $# -gt 0 ]; then
        echo -e "${BLUE}🚀 [Parent] Executing child command: $@${NC}"
        
        # Nếu không keep-alive thì trap dọn dẹp
        if [ "$KEEP_ALIVE" = false ]; then
            trap leeattend_cleanup EXIT INT TERM
        fi

        # Thực thi lệnh
        eval "$@"
    else
        echo -e "\n${YELLOW}⚠️ No child command provided.${NC}"
        if [ "$KEEP_ALIVE" = true ]; then
            echo -e "Environment is alive. Manual cleanup required later with:"
            echo -e "${YELLOW}docker-compose -f docker-compose.test.yml down -v${NC}"
        else
            leeattend_cleanup
        fi
    fi
}

# Nếu script được chạy trực tiếp (không phải bị sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    run_within_env "$@"
fi