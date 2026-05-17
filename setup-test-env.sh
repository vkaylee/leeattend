#!/bin/bash

# ==============================================================================
# LeeAttend Test Environment "Abstract Parent"
# Handles: Postgres Setup, Isolated DBs, Migrations, and Cleanup.
# ==============================================================================

set -euo pipefail

# --- Configuration ---
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PROJECT_ROOT
export COMPOSE_PROJECT_NAME="leeattend-test"
export DOCKER_COMPOSE_CMD="podman-compose -p $COMPOSE_PROJECT_NAME -f $PROJECT_ROOT/docker-compose.yml -f $PROJECT_ROOT/.compose/docker-compose.test.yml"
export LOG_DIR="$PROJECT_ROOT/.test_logs"
mkdir -p "$LOG_DIR"

# Auto-include CI override only in CI environments 
if [[ "${CI:-}" == "true" && -f "$PROJECT_ROOT/docker-compose.ci.yml" ]]; then
    export DOCKER_COMPOSE_CMD="$DOCKER_COMPOSE_CMD -f $PROJECT_ROOT/docker-compose.ci.yml"
fi

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
[ "$NUM_WORKERS" -lt 1 ] && export NUM_WORKERS=1

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
    $DOCKER_COMPOSE_CMD down --remove-orphans </dev/null > /dev/null 2>&1
    podman volume prune -f --filter "label=com.docker.compose.project=$COMPOSE_PROJECT_NAME" --filter "label=com.docker.compose.volume=shared-data" </dev/null > /dev/null 2>&1 || true
    rm -f apiserver/.api_port* webdashboard/.web_port*
    echo -e "${GREEN}✅ Cleanup complete.${NC}"
}

leeattend_setup() {
    echo -e "${BLUE}🏗️ [Parent] Setting up Postgres Environment...${NC}"

    # We need apiserver up to run migrations inside it
    $DOCKER_COMPOSE_CMD up -d db apiserver </dev/null > /dev/null 2>&1

    echo -n "  Waiting for database to be ready (timeout: ${TIMEOUT_DB_WAIT}s)..."
    local db_wait_start
    db_wait_start=$(date +%s)
    until $DOCKER_COMPOSE_CMD exec db pg_isready -U test_user -d test_database </dev/null > /dev/null 2>&1; do
        printf "."
        sleep 1
        local elapsed=$(($(date +%s) - db_wait_start))
        if [ "$elapsed" -ge "$TIMEOUT_DB_WAIT" ]; then
            echo -e "\n${RED}❌ Database wait TIMED OUT after ${TIMEOUT_DB_WAIT}s${NC}"
            exit 124
        fi
    done
    echo -e " ${GREEN}Ready!${NC}"

    echo -e "  Creating isolated databases for $NUM_WORKERS workers..."

    # Clean up zombie test databases from previous runs
    # When nextest kills tests via SIGKILL (timeout), Rust's Drop trait doesn't run,
    # leaving orphan test_db_* databases that consume disk space.
    $DOCKER_COMPOSE_CMD exec -T -e PGPASSWORD="$PGPASSWORD" db psql -U test_user -d test_database -t -c \
        "SELECT 'DROP DATABASE IF EXISTS \"' || datname || '\";' FROM pg_database WHERE datname LIKE 'test_db_%';" \
        </dev/null 2>/dev/null | \
        $DOCKER_COMPOSE_CMD exec -T -i -e PGPASSWORD="$PGPASSWORD" db psql -U test_user -d test_database \
        </dev/null > /dev/null 2>&1 || true

    # Create template database for Rust tests
    $DOCKER_COMPOSE_CMD exec -T -e PGPASSWORD="$PGPASSWORD" db psql -U test_user -d test_database -c "DROP DATABASE IF EXISTS leeattend_test_template;" </dev/null > /dev/null 2>&1 || true
    $DOCKER_COMPOSE_CMD exec -T -e PGPASSWORD="$PGPASSWORD" db psql -U test_user -d test_database -c "CREATE DATABASE leeattend_test_template;" </dev/null > /dev/null 2>&1 || true



    # Internal URLs
    local MAIN_DB_URL="postgres://test_user:test_password@db:5432/test_database?sslmode=disable"
    local TEMPLATE_DB_URL="postgres://test_user:test_password@db:5432/leeattend_test_template?sslmode=disable"

    echo -e "  Running migrations..."
    if ! $DOCKER_COMPOSE_CMD exec -T -e PGPASSWORD="$PGPASSWORD" -e DATABASE_URL="$MAIN_DB_URL" apiserver bash -c "cargo test --test setup_migrations -- run_migrations --ignored --exact" > "$LOG_DIR/parent_mig_main.log" 2>&1; then
        echo -e "${RED}❌ Main migration FAILED. Logs from container:${NC}"
        cat "$LOG_DIR/parent_mig_main.log"
        exit 1
    fi
    if ! $DOCKER_COMPOSE_CMD exec -T -e PGPASSWORD="$PGPASSWORD" -e DATABASE_URL="$TEMPLATE_DB_URL" apiserver bash -c "cargo test --test setup_migrations -- run_migrations --ignored --exact" > "$LOG_DIR/parent_mig_template.log" 2>&1; then
        echo -e "${RED}❌ Template migration FAILED. Logs from container:${NC}"
        cat "$LOG_DIR/parent_mig_template.log"
        exit 1
    fi

    # Ensure no connections hang on template for CREATE DATABASE TEMPLATE usage
    $DOCKER_COMPOSE_CMD exec -T -e PGPASSWORD="$PGPASSWORD" db psql -U test_user -d test_database -c "REVOKE CONNECT ON DATABASE leeattend_test_template FROM public;" </dev/null > /dev/null 2>&1
    $DOCKER_COMPOSE_CMD exec -T -e PGPASSWORD="$PGPASSWORD" db psql -U test_user -d test_database -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'leeattend_test_template' AND pid <> pg_backend_pid();" </dev/null > /dev/null 2>&1



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
        echo -e "${BLUE}🚀 [Parent] Executing child command: $*${NC}"
        
        # Nếu không keep-alive thì trap dọn dẹp
        if [ "$KEEP_ALIVE" = false ]; then
            trap leeattend_cleanup EXIT INT TERM
        fi

        # If the command starts with leeattend_run_, we need to source the scripts
        # because functions are not exported to the subshell eval.
        local cmd="$1"
        if [[ "$cmd" == leeattend_run_* ]]; then
            # shellcheck disable=SC1091
            source "$PROJECT_ROOT/scripts/test-utils.sh"
            # shellcheck disable=SC1091
            source "$PROJECT_ROOT/scripts/test-lint.sh"
            # shellcheck disable=SC1091
            source "$PROJECT_ROOT/scripts/test-unit.sh"
            # shellcheck disable=SC1091
            source "$PROJECT_ROOT/scripts/test-integration.sh"
        fi

        # Thực thi lệnh
        "$@"
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
