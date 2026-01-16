#!/bin/bash
set -euo pipefail
SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SOURCE_DIR/test-utils.sh"

run_parallel_ordered "Phase 1 (Unit Tests)" \
    "1|apiserver|apiserver|SQLX_OFFLINE=true cargo test --lib" \
    "2|agent-main|agent-main|SQLX_OFFLINE=true cargo test --lib" \
    "3|agent-zk-worker|agent-zk-worker|venv/bin/pytest" \
    "4|webdashboard|webdashboard|npm run test"
