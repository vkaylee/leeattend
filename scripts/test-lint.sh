#!/bin/bash

# ==============================================================================
# ⚠️ INTERNAL MODULE — DO NOT RUN DIRECTLY
# Sourced by: ./run-tests.sh
# ==============================================================================
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "❌ This script is an internal module. Do not run directly."
    echo "👉 Use: ./run-tests.sh --api  (or --all)"
    exit 1
fi
# ==============================================================================

# LeeAttend Linting Methods
# ==============================================================================

leeattend_run_lint() {
    local filter="${1:-}"
    
    local BACKEND_CMD="cargo clippy --workspace -- -D warnings"
    local WEB_LINT_CMD="npm run lint"
    local WEB_TYPECHECK_CMD="npm run type-check"
    local SYNC_API_CMD="./scripts/sync-api.sh"

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # Lint requires DB with migrations (sqlx verifies queries at compile time)
        BACKEND_CMD="$DOCKER_COMPOSE_CMD exec -T -w /workspace apiserver_test bash -c \"cargo clippy --workspace -- -D warnings\" </dev/null"
        WEB_LINT_CMD="$DOCKER_COMPOSE_CMD exec -T web_test npm run lint </dev/null"
        WEB_TYPECHECK_CMD="$DOCKER_COMPOSE_CMD exec -T web_test npm run type-check </dev/null"
        
        # API Sync via shared volume between apiserver_test and web_test:
        SYNC_API_CMD="$DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c \"cargo run -- gen-openapi > /workspace/shared/openapi.json\" </dev/null && $DOCKER_COMPOSE_CMD exec -T web_test bash -c \"npx openapi-typescript /workspace/shared/openapi.json -o ./src/types/api-generated.ts\" </dev/null"
    fi

    run_parallel_ordered "Phase 0 Linting" "$filter" \
        "0|api-sync|webdashboard|$SYNC_API_CMD" \
        "1|rust-backend-clippy|apiserver|$BACKEND_CMD" \
        "3|webdashboard-lint|webdashboard|$WEB_LINT_CMD" \
        "4|webdashboard-typecheck|webdashboard|$WEB_TYPECHECK_CMD"
}
