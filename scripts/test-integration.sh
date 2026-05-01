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

    # nextest uses substrings for matching tests.
    local NO_TESTS_FLAG="pass"

    # Integration tests use isolated databases (db_{uuid} from template)
    # so they can safely run in parallel. Thread count is set in nextest.toml (num-cpus).
    echo -e "${YELLOW}⚙️ Integration mode: Parallel execution enabled (nextest.toml → num-cpus).${NC}"

    # If pattern is the same as filter (no space), then pattern is empty
    if [[ "$COMPONENT_FILTER" == "$TEST_PATTERN" ]]; then
        TEST_PATTERN=""
    fi

    # Default URL for internal tests
    export DATABASE_URL="postgres://test_user:test_password@db:5432/leeattend_test_template"

    local BACKEND_INT_CMD="cargo nextest run --workspace --test '*' $TEST_PATTERN --no-tests=$NO_TESTS_FLAG"
    
    # For Playwright, we use -g for grep if pattern is provided
    local PW_ARGS=""
    if [ -n "$TEST_PATTERN" ]; then
        PW_ARGS="-g \"$TEST_PATTERN\""
    fi
    local PW_CMD="npx playwright test $PW_ARGS --workers 1"

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # DOCKER_DB_URL: Used for sqlx compile-time macro checks (needs schema)
        local DOCKER_DB_URL="postgres://test_user:test_password@db:5432/leeattend_test_template"
        # API_SERVER_DB_URL: Used for the background API server process.
        # MUST NOT use the template DB — holding connections blocks CREATE DATABASE ... TEMPLATE.
        local API_SERVER_DB_URL="postgres://test_user:test_password@db:5432/test_database"
        
        if [[ "$COMPONENT_FILTER" != *"web"* ]]; then
            # START API SERVER IN BACKGROUND FOR AGENT INTEGRATION TESTS
            # Use direct `podman exec` (not podman-compose exec) for speed —
            # podman-compose exec reinitializes compose state on every call,
            # adding ~2-3s overhead per invocation. With 60 health checks,
            # that's 2-3 minutes wasted just on overhead.
            local _apiserver_ctr="${COMPOSE_PROJECT_NAME}_apiserver_1"
            echo -e "  ${BLUE}🚀 Starting API Server in background inside Docker...${NC}"
            podman exec -d -e DATABASE_URL=$API_SERVER_DB_URL "$_apiserver_ctr" bash -c "cargo run --bin apiserver > /workspace/apiserver/apiserver.log 2>&1" </dev/null
            
            # Wait for server to be ready (direct podman exec for speed)
            echo -n "  Waiting for API Server (timeout: 600s)..."
            for i in {1..600}; do
                if ! podman container exists "$_apiserver_ctr" 2>/dev/null; then
                    echo -e "\n  ${RED}❌ Container $_apiserver_ctr unexpectedly died or was removed!${NC}"
                    break
                fi
                if podman exec "$_apiserver_ctr" curl -s http://localhost:3000/api/health </dev/null > /dev/null 2>&1; then
                    echo " Ready!"
                    break
                fi
                if [[ $i -eq 600 ]]; then
                    echo -e "\n  ${RED}❌ API Server failed to start! Logs:${NC}"
                    podman exec "$_apiserver_ctr" tail -20 /workspace/apiserver/apiserver.log </dev/null 2>&1
                fi
                echo -n "."
                sleep 1
            done
        fi

        local THREADS=${NEXTEST_TEST_THREADS:-1}
        BACKEND_INT_CMD="$DOCKER_COMPOSE_CMD exec -T -e DATABASE_URL=$DOCKER_DB_URL -e NEXTEST_TEST_THREADS=$THREADS -w /workspace apiserver bash -c \"cargo nextest run --workspace --test '*' $TEST_PATTERN --no-tests=$NO_TESTS_FLAG\" </dev/null"
        PW_CMD="$DOCKER_COMPOSE_CMD exec -T webdashboard bash -c \"npx playwright test $PW_ARGS --workers 1\" </dev/null"
    fi

    if [[ "$COMPONENT_FILTER" != *"web"* ]]; then
        echo -e "  Pre-compiling backend for integration tests (timeout: ${TIMEOUT_BUILD}s)..."
        local prebuild_exit=0
        if [[ "${USE_DOCKER:-}" == "true" ]]; then
            $DOCKER_COMPOSE_CMD exec -T -e DATABASE_URL=$DOCKER_DB_URL -w /workspace apiserver bash -c "cargo test --workspace --no-run -j $CARGO_BUILD_JOBS" </dev/null > "$LOG_DIR/prebuild.log" 2>&1
            prebuild_exit=$?
        else
            run_with_timeout "$TIMEOUT_BUILD" bash -c "cd '$PROJECT_ROOT' && DATABASE_URL=$DATABASE_URL cargo test --workspace --no-run -j $CARGO_BUILD_JOBS" > "$LOG_DIR/prebuild.log" 2>&1
            prebuild_exit=$?
        fi
        if [[ $prebuild_exit -ne 0 ]]; then
            echo -e "  ${RED}❌ Pre-compilation FAILED (exit code: $prebuild_exit)${NC}"
            echo -e "  ${RED}── Build Error Log ──${NC}"
            # Show only the error lines, not the full compile output
            tail -30 <<< "$(grep -E '(^error|^\s+-->|could not compile)' "$LOG_DIR/prebuild.log" 2>/dev/null || true)"
            return $prebuild_exit
        fi
    fi

    # Note: Individual task timeouts are handled by run_parallel_ordered via TIMEOUT_INTEGRATION
    run_parallel_ordered "Phase 3 (Integration & E2E)" "$COMPONENT_FILTER" \
        "1|rust-backend-int|apiserver|$BACKEND_INT_CMD" \
        "3|playwright-e2e|webdashboard|$PW_CMD"
}
