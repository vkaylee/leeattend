#!/bin/bash
set -euo pipefail
SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SOURCE_DIR/test-utils.sh"

APISERVER_CMD="SQLX_OFFLINE=true cargo clippy -- -D warnings"
AGENT_MAIN_CMD="SQLX_OFFLINE=true cargo clippy -- -D warnings"
WEB_LINT_CMD="npm run lint"
WEB_TYPECHECK_CMD="npm run type-check"
SYNC_API_CMD="./scripts/sync-api.sh"

if [[ "${USE_DOCKER:-}" == "true" ]]; then
    APISERVER_CMD="$DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c \"SQLX_OFFLINE=true cargo clippy -- -D warnings\""
    AGENT_MAIN_CMD="$DOCKER_COMPOSE_CMD exec -T agent_test bash -c \"SQLX_OFFLINE=true cargo clippy -- -D warnings\""
    WEB_LINT_CMD="$DOCKER_COMPOSE_CMD exec -T web_test npm run lint"
    WEB_TYPECHECK_CMD="$DOCKER_COMPOSE_CMD exec -T web_test npm run type-check"
    
    # API Sync via shared volume between apiserver_test and web_test:
    # 1. Generate JSON in apiserver container → /workspace/shared/openapi.json
    # 2. Generate TS types in web container from shared volume
    SYNC_API_CMD="$DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c \"SQLX_OFFLINE=true cargo run -- gen-openapi > /workspace/shared/openapi.json\" && $DOCKER_COMPOSE_CMD exec -T web_test bash -c \"npx openapi-typescript /workspace/shared/openapi.json -o ./src/types/api-generated.ts\""
fi

run_parallel_ordered "Phase 0 Linting" "${1:-}" \
    "0|api-sync|webdashboard|$SYNC_API_CMD" \
    "1|apiserver-clippy|apiserver|$APISERVER_CMD" \
    "2|agent-main-clippy|agent-main|$AGENT_MAIN_CMD" \
    "3|webdashboard-lint|webdashboard|$WEB_LINT_CMD" \
    "4|webdashboard-typecheck|webdashboard|$WEB_TYPECHECK_CMD"
