#!/bin/bash

# ==============================================================================
# ⚠️ FOR AI AGENTS: Use ./run-tests.sh instead of running this script directly!
# ==============================================================================
# This script is a module sourced by run-tests.sh
# Run: ./run-tests.sh --api --integration   or   ./run-tests.sh --all
# ==============================================================================

# LeeAttend Integration Testing Methods
# ==============================================================================

leeattend_run_integration() {
    local filter_input="${1:-}"
    
    # Extract filter and optional pattern
    local COMPONENT_FILTER=$(echo "$filter_input" | cut -d':' -f1)
    local TEST_PATTERN=$(echo "$filter_input" | cut -d':' -f2-)

    # Force sequential execution for integration tests
    export NUM_WORKERS=1
    echo -e "${YELLOW}⚠️ Integration mode: Overriding NUM_WORKERS to 1 for sequential execution.${NC}"

    # If pattern is the same as filter (no space), then pattern is empty
    if [[ "$COMPONENT_FILTER" == "$TEST_PATTERN" ]]; then
        TEST_PATTERN=""
    fi

    # Default URL for internal tests
    export DATABASE_URL="postgres://test_user:test_password@test_db:5432/leeattend_test_template"

    local APISERVER_INT_CMD="cargo nextest run --test '*' $TEST_PATTERN --no-tests=pass --test-threads 1"
    
    # For Playwright, we use -g for grep if pattern is provided
    local PW_ARGS=""
    if [ -n "$TEST_PATTERN" ]; then
        PW_ARGS="-g \"$TEST_PATTERN\""
    fi
    local PW_CMD="npx playwright test $PW_ARGS --workers 1"

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # In Docker, we connect to 'test_db' service and template DB
        local DOCKER_DB_URL="postgres://test_user:test_password@test_db:5432/leeattend_test_template"
        
        if [[ "$COMPONENT_FILTER" != *"web"* ]]; then
            # START API SERVER IN BACKGROUND FOR AGENT INTEGRATION TESTS
            echo -e "  ${BLUE}🚀 Starting API Server in background inside Docker...${NC}"
            $DOCKER_COMPOSE_CMD exec -d -e DATABASE_URL=$DOCKER_DB_URL apiserver_test bash -c "cargo run --bin apiserver > /workspace/apiserver/apiserver.log 2>&1"
            
            # Wait for server to be ready
            echo -n "  Waiting for API Server (timeout: 60s)..."
            for i in {1..60}; do
                if $DOCKER_COMPOSE_CMD exec -T apiserver_test curl -s http://localhost:3000/api/health > /dev/null; then
                    echo " Ready!"
                    break
                fi
                echo -n "."
                sleep 1
            done
        fi

        APISERVER_INT_CMD="$DOCKER_COMPOSE_CMD exec -T -e DATABASE_URL=$DOCKER_DB_URL apiserver_test bash -c \"cargo nextest run --test '*' $TEST_PATTERN --no-tests=pass --test-threads 1\""
        PW_CMD="$DOCKER_COMPOSE_CMD exec -T web_test bash -c \"npx playwright test $PW_ARGS --workers 1\""
    fi

    if [[ "$COMPONENT_FILTER" != *"web"* ]]; then
        echo -e "  Pre-compiling backend for integration tests (timeout: ${TIMEOUT_BUILD}s)..."
        if [[ "${USE_DOCKER:-}" == "true" ]]; then
            $DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c "SQLX_OFFLINE=false cargo test --no-run -j $CARGO_BUILD_JOBS" > "$LOG_DIR/prebuild.log" 2>&1
        else
            run_with_timeout "$TIMEOUT_BUILD" bash -c "cd '$PROJECT_ROOT/apiserver' && SQLX_OFFLINE=false cargo test --no-run -j $CARGO_BUILD_JOBS" > "$LOG_DIR/prebuild.log" 2>&1
        fi
    fi

    # Note: Individual task timeouts are handled by run_parallel_ordered via TIMEOUT_INTEGRATION
    run_parallel_ordered "Phase 3 (Integration & E2E)" "$COMPONENT_FILTER" \
        "1|apiserver-int|apiserver|$APISERVER_INT_CMD" \
        "2|playwright-e2e|webdashboard|$PW_CMD"
}
