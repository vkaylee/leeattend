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

# Default URL for host-based tests
export DATABASE_URL="postgres://test_user:test_password@localhost:5433/leeattend_test_template"

APISERVER_INT_CMD="cargo test --test '*' $TEST_PATTERN -- --test-threads $NUM_WORKERS"
PW_CMD="npx playwright test ${PW_ARGS:-} --workers $NUM_WORKERS"

if [[ "${USE_DOCKER:-}" == "true" ]]; then
    # In Docker, we connect to 'test_db' service and template DB
    DOCKER_DB_URL="postgres://test_user:test_password@test_db:5432/leeattend_test_template"
    
    # START API SERVER IN BACKGROUND FOR AGENT INTEGRATION TESTS
    # We need to start the server inside the container so the Agent can talk to it
    echo -e "  ${BLUE}🚀 Starting API Server in background inside Docker...${NC}"
    $DOCKER_COMPOSE_CMD exec -d -e DATABASE_URL=$DOCKER_DB_URL apiserver_test bash -c "cargo run --bin apiserver > /app/apiserver.log 2>&1"
    
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

    APISERVER_INT_CMD="$DOCKER_COMPOSE_CMD exec -T -e DATABASE_URL=$DOCKER_DB_URL apiserver_test bash -c \"cargo test --test '*' $TEST_PATTERN -- --test-threads $NUM_WORKERS\""
    # Web E2E usually runs better in its own container or on host with specific display setup
    PW_CMD="$DOCKER_COMPOSE_CMD exec -T web_test bash -c \"npx playwright test ${PW_ARGS:-} --workers $NUM_WORKERS\""
fi

echo -e "  Pre-compiling backend for integration tests (timeout: ${TIMEOUT_BUILD}s)..."
run_with_timeout "$TIMEOUT_BUILD" bash -c "cd '$PROJECT_ROOT/apiserver' && cargo test --no-run -j ${CARGO_BUILD_JOBS:-2}" > "$LOG_DIR/prebuild.log" 2>&1
prebuild_exit_code=$?

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
