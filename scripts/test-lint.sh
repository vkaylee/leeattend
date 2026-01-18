#!/bin/bash
set -euo pipefail
SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SOURCE_DIR/test-utils.sh"

run_parallel_ordered "Phase 0 (Linting)" \
    "1|apiserver-clippy|apiserver|SQLX_OFFLINE=true cargo clippy -- -D warnings" \
    "2|agent-main-clippy|agent-main|SQLX_OFFLINE=true cargo clippy -- -D warnings" \
    "3|webdashboard-lint|webdashboard|npm run lint" \
    "4|webdashboard-typecheck|webdashboard|npm run type-check"
