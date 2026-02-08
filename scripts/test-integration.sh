#!/bin/bash
set -euo pipefail
SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SOURCE_DIR/test-utils.sh"

# Extract filter and optional pattern
INPUT_FILTER="${1:-}"
COMPONENT_FILTER=$(echo "$INPUT_FILTER" | cut -d':' -f1)
TEST_PATTERN=$(echo "$INPUT_FILTER" | cut -d':' -f2-)

# If pattern is the same as filter (no space), then pattern is empty
if [[ "$COMPONENT_FILTER" == "$TEST_PATTERN" ]]; then
    TEST_PATTERN=""
fi

# Default URL for internal tests
export DATABASE_URL="postgres://test_user:test_password@test_db:5432/leeattend_test_template"

APISERVER_INT_CMD="cargo nextest run --test '*' $TEST_PATTERN --no-tests=pass"
PW_CMD="npx playwright test ${PW_ARGS:-} --workers $NUM_WORKERS"

if [[ "${USE_DOCKER:-}" == "true" ]]; then
    # In Docker, we connect to 'test_db' service and template DB
    DOCKER_DB_URL="postgres://test_user:test_password@test_db:5432/leeattend_test_template"
    
    if [[ "$COMPONENT_FILTER" != *"web"* ]]; then
        # START API SERVER IN BACKGROUND FOR AGENT INTEGRATION TESTS
        echo -e "  ${BLUE}🚀 Starting API Server in background inside Docker...${NC}"
        $DOCKER_COMPOSE_CMD exec -d -e DATABASE_URL=$DOCKER_DB_URL apiserver_test bash -c "cargo run --bin apiserver > /workspace/apiserver/apiserver.log 2>&1"
        
        # Wait for server to be ready
        echo -n "  Waiting for API Server (timeout: 60s)..."
        for i in {1..60}; do
            if $DOCKER_COMPOSE_CMD exec -T apiserver_test curl -s http://localhost:3001/health > /dev/null; then
                echo " Ready!"
                break
            fi
            echo -n "."
            sleep 1
        done
    fi

    APISERVER_INT_CMD="$DOCKER_COMPOSE_CMD exec -T -e DATABASE_URL=$DOCKER_DB_URL apiserver_test bash -c \"cargo nextest run --test '*' $TEST_PATTERN --no-tests=pass\""
    PW_CMD="$DOCKER_COMPOSE_CMD exec -T web_test bash -c \"npx playwright test ${PW_ARGS:-} --workers 1\""
fi

if [[ "$COMPONENT_FILTER" != *"web"* ]]; then
    echo -e "  Pre-compiling backend for integration tests (timeout: ${TIMEOUT_BUILD}s)..."
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        $DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c "SQLX_OFFLINE=true cargo test --no-run -j ${CARGO_BUILD_JOBS:-2}" > "$LOG_DIR/prebuild.log" 2>&1
    else
        run_with_timeout "$TIMEOUT_BUILD" bash -c "cd '$PROJECT_ROOT/apiserver' && SQLX_OFFLINE=true cargo test --no-run -j ${CARGO_BUILD_JOBS:-2}" > "$LOG_DIR/prebuild.log" 2>&1
    fi
    prebuild_exit_code=$?
fi

export RUST_TEST_THREADS=$NUM_WORKERS

# For Playwright, we use -g for grep if pattern is provided
PW_ARGS=""
if [ -n "$TEST_PATTERN" ]; then
    PW_ARGS="-g \"$TEST_PATTERN\""
fi

# Note: Individual task timeouts are handled by run_parallel_ordered via TIMEOUT_INTEGRATION
run_parallel_ordered "Phase 3 (Integration & E2E)" "$COMPONENT_FILTER" \
    "1|apiserver-int|apiserver|$APISERVER_INT_CMD" \
    "2|playwright-e2e|webdashboard|$PW_CMD"
