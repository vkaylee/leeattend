#!/bin/bash
set -euo pipefail
SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SOURCE_DIR/test-utils.sh"

APISERVER_CMD="SQLX_OFFLINE=true cargo clippy -- -D warnings"
AGENT_MAIN_CMD="SQLX_OFFLINE=true cargo clippy -- -D warnings"
WEB_LINT_CMD="npm run lint"
WEB_TYPECHECK_CMD="npm run type-check"

if [[ "${USE_DOCKER:-}" == "true" ]]; then
    APISERVER_CMD="$DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c \"SQLX_OFFLINE=true cargo clippy -- -D warnings\""
    AGENT_MAIN_CMD="$DOCKER_COMPOSE_CMD exec -T agent_test bash -c \"SQLX_OFFLINE=true cargo clippy -- -D warnings\""
    WEB_LINT_CMD="$DOCKER_COMPOSE_CMD exec -T web_test npm run lint"
    WEB_TYPECHECK_CMD="$DOCKER_COMPOSE_CMD exec -T web_test npm run type-check"
fi

run_parallel_ordered "Phase 0 (Linting)" "${1:-}" \
    "1|apiserver-clippy|apiserver|$APISERVER_CMD" \
    "2|agent-main-clippy|agent-main|$AGENT_MAIN_CMD" \
    "3|webdashboard-lint|webdashboard|$WEB_LINT_CMD" \
    "4|webdashboard-typecheck|webdashboard|$WEB_TYPECHECK_CMD"
