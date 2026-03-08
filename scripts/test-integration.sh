#!/bin/bash

# ==============================================================================
# ⚠️ INTERNAL MODULE — DO NOT RUN DIRECTLY
# Sourced by: ./run-tests.sh
# ==============================================================================
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "❌ This script is an internal module. Do not run directly."
    echo "👉 Use: ./run-tests.sh --integration  (or --all)"
    exit 1
fi
# ==============================================================================

# LeeAttend Integration Testing Methods
# ==============================================================================

leeattend_run_integration() {
    local filter_input="${1:-}"
    
    # Extract filter and optional pattern
    local COMPONENT_FILTER=$(echo "$filter_input" | cut -d':' -f1)
    local TEST_PATTERN=$(echo "$filter_input" | cut -d':' -f2-)

    # Integration tests use isolated databases (test_db_{uuid} from template)
    # so they can safely run in parallel. Thread count is set in nextest.toml (num-cpus).
    echo -e "${YELLOW}⚙️ Integration mode: Parallel execution enabled (nextest.toml → num-cpus).${NC}"

    # If pattern is the same as filter (no space), then pattern is empty
    if [[ "$COMPONENT_FILTER" == "$TEST_PATTERN" ]]; then
        TEST_PATTERN=""
    fi

    # Default URL for internal tests
    export DATABASE_URL="postgres://test_user:test_password@test_db:5432/leeattend_test_template"

    local APISERVER_INT_CMD="cargo nextest run --test '*' $TEST_PATTERN --no-tests=pass"
    
    # For Playwright, we use -g for grep if pattern is provided
    local PW_ARGS=""
    if [ -n "$TEST_PATTERN" ]; then
        PW_ARGS="-g \"$TEST_PATTERN\""
    fi
    local PW_CMD="npx playwright test $PW_ARGS --workers 1"

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # DOCKER_DB_URL: Used for sqlx compile-time macro checks (needs schema)
        local DOCKER_DB_URL="postgres://test_user:test_password@test_db:5432/leeattend_test_template"
        # API_SERVER_DB_URL: Used for the background API server process.
        # MUST NOT use the template DB — holding connections blocks CREATE DATABASE ... TEMPLATE.
        local API_SERVER_DB_URL="postgres://test_user:test_password@test_db:5432/test_database"
        
        if [[ "$COMPONENT_FILTER" != *"web"* ]]; then
            # START API SERVER IN BACKGROUND FOR AGENT INTEGRATION TESTS
            # Use direct `podman exec` (not podman-compose exec) for speed —
            # podman-compose exec reinitializes compose state on every call,
            # adding ~2-3s overhead per invocation. With 60 health checks,
            # that's 2-3 minutes wasted just on overhead.
            local _apiserver_ctr="${COMPOSE_PROJECT_NAME}_apiserver_test_1"
            echo -e "  ${BLUE}🚀 Starting API Server in background inside Docker...${NC}"
            podman exec -d -e DATABASE_URL=$API_SERVER_DB_URL "$_apiserver_ctr" bash -c "cargo run --bin apiserver > /workspace/apiserver/apiserver.log 2>&1" </dev/null
            
            # Wait for server to be ready (direct podman exec for speed)
            echo -n "  Waiting for API Server (timeout: 120s)..."
            for i in {1..120}; do
                if podman exec "$_apiserver_ctr" curl -s http://localhost:3000/api/health </dev/null > /dev/null 2>&1; then
                    echo " Ready!"
                    break
                fi
                if [[ $i -eq 120 ]]; then
                    echo -e "\n  ${RED}❌ API Server failed to start! Logs:${NC}"
                    podman exec "$_apiserver_ctr" cat /workspace/apiserver/apiserver.log </dev/null 2>&1 | tail -20
                fi
                echo -n "."
                sleep 1
            done
        fi

        APISERVER_INT_CMD="$DOCKER_COMPOSE_CMD exec -T -e DATABASE_URL=$DOCKER_DB_URL apiserver_test bash -c \"cargo nextest run --test '*' $TEST_PATTERN --no-tests=pass\" </dev/null"
        PW_CMD="$DOCKER_COMPOSE_CMD exec -T web_test bash -c \"npx playwright test $PW_ARGS --workers 1\" </dev/null"
    fi

    if [[ "$COMPONENT_FILTER" != *"web"* ]]; then
        echo -e "  Pre-compiling backend for integration tests (timeout: ${TIMEOUT_BUILD}s)..."
        local prebuild_exit=0
        if [[ "${USE_DOCKER:-}" == "true" ]]; then
            $DOCKER_COMPOSE_CMD exec -T -e DATABASE_URL=$DOCKER_DB_URL apiserver_test bash -c "cargo test --no-run -j $CARGO_BUILD_JOBS" </dev/null > "$LOG_DIR/prebuild.log" 2>&1
            prebuild_exit=$?
        else
            run_with_timeout "$TIMEOUT_BUILD" bash -c "cd '$PROJECT_ROOT/apiserver' && DATABASE_URL=$DATABASE_URL cargo test --no-run -j $CARGO_BUILD_JOBS" > "$LOG_DIR/prebuild.log" 2>&1
            prebuild_exit=$?
        fi
        if [[ $prebuild_exit -ne 0 ]]; then
            echo -e "  ${RED}❌ Pre-compilation FAILED (exit code: $prebuild_exit)${NC}"
            echo -e "  ${RED}── Build Error Log ──${NC}"
            # Show only the error lines, not the full compile output
            grep -E '(^error|^\s+-->|could not compile)' "$LOG_DIR/prebuild.log" | tail -30
            return $prebuild_exit
        fi
    fi

    # Note: Individual task timeouts are handled by run_parallel_ordered via TIMEOUT_INTEGRATION
    run_parallel_ordered "Phase 3 (Integration & E2E)" "$COMPONENT_FILTER" \
        "1|apiserver-int|apiserver|$APISERVER_INT_CMD" \
        "2|playwright-e2e|webdashboard|$PW_CMD"
}
