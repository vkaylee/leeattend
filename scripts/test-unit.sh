#!/bin/bash

# ==============================================================================
# LeeAttend Unit Testing Methods
# ==============================================================================

leeattend_run_unit() {
    local filter_input="${1:-}"
    
    # Extract filter and optional pattern
    local COMPONENT_FILTER=$(echo "$filter_input" | cut -d':' -f1)
    local TEST_PATTERN=$(echo "$filter_input" | cut -d':' -f2-)

    # If pattern is the same as filter (no space), then pattern is empty
    if [ "$COMPONENT_FILTER" == "$TEST_PATTERN" ]; then
        TEST_PATTERN=""
    fi

    local APISERVER_CMD="SQLX_OFFLINE=false cargo nextest run --lib $TEST_PATTERN --no-tests=pass"
    local AGENT_MAIN_CMD="SQLX_OFFLINE=false cargo nextest run --lib $TEST_PATTERN --no-tests=pass"
    local WEB_CMD="npm run test -- $TEST_PATTERN --passWithNoTests"

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        APISERVER_CMD="$DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c \"SQLX_OFFLINE=false cargo nextest run --lib $TEST_PATTERN --no-tests=pass || [ \$? -eq 4 ]\""
        AGENT_MAIN_CMD="$DOCKER_COMPOSE_CMD exec -T -e CENTRAL_SERVER_URL=ws://unit-test-invalid:3000 agent_test bash -c \"SQLX_OFFLINE=false cargo nextest run --lib $TEST_PATTERN --no-tests=pass || [ \$? -eq 4 ]\""
        WEB_CMD="$DOCKER_COMPOSE_CMD exec -T web_test npm run test -- $TEST_PATTERN --passWithNoTests"
    fi

    run_parallel_ordered "Phase 1 (Unit Tests)" "$COMPONENT_FILTER" \
        "1|apiserver|apiserver|$APISERVER_CMD" \
        "2|agent-main|agent-main|$AGENT_MAIN_CMD" \
        "4|webdashboard|webdashboard|$WEB_CMD"
}
