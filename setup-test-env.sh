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

# --- Timeout Configuration ---
export TIMEOUT_DB_WAIT=${TIMEOUT_DB_WAIT:-120}     # 2 min for DB ready check
export TIMEOUT_MIGRATION=${TIMEOUT_MIGRATION:-120} # 2 min for migrations
export TIMEOUT_SQLX_PREPARE=${TIMEOUT_SQLX_PREPARE:-180} # 3 min for sqlx prepare

# Common Variables
export DATABASE_URL="postgres://test_user:test_password@test_db:5432/test_database"
export PGPASSWORD="test_password"
export CARGO_BUILD_JOBS=2
CPU_CORES=$(nproc)
export NUM_WORKERS=${NUM_WORKERS:-$(( CPU_CORES / 4 ))}
[ $NUM_WORKERS -lt 1 ] && export NUM_WORKERS=1

# ANSI Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# --- Helper: Run with timeout ---
run_with_timeout() {
    local timeout_sec=$1
    shift
    timeout --signal=KILL "${timeout_sec}s" "$@"
}

# --- Methods (The "Interface" Logic) ---

leeattend_cleanup() {
    echo -e "\n${YELLOW}🧹 [Parent] Cleaning up test environment...${NC}"
    docker compose -f "$COMPOSE_FILE" down -v > /dev/null 2>&1
    rm -f apiserver/.api_port* webdashboard/.web_port*
    echo -e "${GREEN}✅ Cleanup complete.${NC}"
}

leeattend_setup() {
    echo -e "${BLUE}🏗️ [Parent] Setting up Postgres Environment...${NC}"

    # We need apiserver_test up to run migrations inside it
    docker compose -f "$COMPOSE_FILE" up -d test_db apiserver_test > /dev/null 2>&1

    echo -n "  Waiting for database to be ready (timeout: ${TIMEOUT_DB_WAIT}s)..."
    local db_wait_start=$(date +%s)
    until docker compose -f "$COMPOSE_FILE" exec test_db pg_isready -U test_user -d test_database > /dev/null 2>&1; do
        printf "."
        sleep 1
        local elapsed=$(($(date +%s) - db_wait_start))
        if [ $elapsed -ge $TIMEOUT_DB_WAIT ]; then
            echo -e "\n${RED}❌ Database wait TIMED OUT after ${TIMEOUT_DB_WAIT}s${NC}"
            exit 124
        fi
    done
    echo -e " ${GREEN}Ready!${NC}"

    echo -e "  Creating isolated databases for $NUM_WORKERS workers..."

    # Create template database for Rust tests
    docker compose -f "$COMPOSE_FILE" exec -e PGPASSWORD="$PGPASSWORD" test_db psql -U test_user -d test_database -c "DROP DATABASE IF EXISTS leeattend_test_template;" > /dev/null 2>&1 || true
    docker compose -f "$COMPOSE_FILE" exec -e PGPASSWORD="$PGPASSWORD" test_db psql -U test_user -d test_database -c "CREATE DATABASE leeattend_test_template;" > /dev/null 2>&1 || true

    for i in $(seq 0 $((NUM_WORKERS-1))); do
        docker compose -f "$COMPOSE_FILE" exec -e PGPASSWORD="$PGPASSWORD" test_db psql -U test_user -d test_database -c "DROP DATABASE IF EXISTS test_database_e2e_$i;" > /dev/null 2>&1 || true
        docker compose -f "$COMPOSE_FILE" exec -e PGPASSWORD="$PGPASSWORD" test_db psql -U test_user -d test_database -c "CREATE DATABASE test_database_e2e_$i;" > /dev/null 2>&1 || true
    done

    # Internal URLs
    local MAIN_DB_URL="postgres://test_user:test_password@test_db:5432/test_database?sslmode=disable"
    local TEMPLATE_DB_URL="postgres://test_user:test_password@test_db:5432/leeattend_test_template?sslmode=disable"

    echo -e "  Running migrations..."
    if ! docker compose -f "$COMPOSE_FILE" exec -T -e PGPASSWORD="$PGPASSWORD" apiserver_test bash -c "sqlx migrate run --database-url '$MAIN_DB_URL' > .test_logs/parent_mig_main.log 2>&1"; then
        echo -e "${RED}❌ Main migration FAILED. Logs from container:${NC}"
        docker compose -f "$COMPOSE_FILE" exec -T apiserver_test cat .test_logs/parent_mig_main.log
        exit 1
    fi
    if ! docker compose -f "$COMPOSE_FILE" exec -T -e PGPASSWORD="$PGPASSWORD" apiserver_test bash -c "sqlx migrate run --database-url '$TEMPLATE_DB_URL' > .test_logs/parent_mig_template.log 2>&1"; then
        echo -e "${RED}❌ Template migration FAILED. Logs from container:${NC}"
        docker compose -f "$COMPOSE_FILE" exec -T apiserver_test cat .test_logs/parent_mig_template.log
        exit 1
    fi

    echo -e "  Preparing SQLx cache..."
    docker compose -f "$COMPOSE_FILE" exec -T -e PGPASSWORD="$PGPASSWORD" -e DATABASE_URL="$TEMPLATE_DB_URL" apiserver_test bash -c "cargo sqlx prepare -- --all-targets > .test_logs/parent_sqlx_prepare.log 2>&1" || echo -e "${YELLOW}⚠️ SQLx prepare failed (non-fatal).${NC}"

    # Ensure no connections hang on template for CREATE DATABASE TEMPLATE usage
    docker compose -f "$COMPOSE_FILE" exec -T -e PGPASSWORD="$PGPASSWORD" test_db psql -U test_user -d test_database -c "REVOKE CONNECT ON DATABASE leeattend_test_template FROM public;" > /dev/null 2>&1
    docker compose -f "$COMPOSE_FILE" exec -T -e PGPASSWORD="$PGPASSWORD" test_db psql -U test_user -d test_database -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'leeattend_test_template' AND pid <> pg_backend_pid();" > /dev/null 2>&1

    for i in $(seq 0 $((NUM_WORKERS-1))); do
        local E2E_DB_URL="postgres://test_user:test_password@test_db:5432/test_database_e2e_$i?sslmode=disable"
        if ! docker compose -f "$COMPOSE_FILE" exec -T -e PGPASSWORD="$PGPASSWORD" apiserver_test bash -c "sqlx migrate run --database-url '$E2E_DB_URL' > .test_logs/parent_mig_e2e_$i.log 2>&1"; then
            echo -e "${RED}❌ E2E DB $i migration FAILED.${NC}"
            exit 1
        fi
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
        echo -e "${YELLOW}⚠️ No child command provided. Environment is up. Press Ctrl+C to cleanup.${NC}"
        # Keep alive until Ctrl+C
        while true; do sleep 1; done
    fi
}

# Auto-execute if script is not sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    run_within_env "$@"
fi
