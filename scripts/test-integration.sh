#!/bin/bash
set -euo pipefail
SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SOURCE_DIR/test-utils.sh"

# Note: Integration tests require the DB environment to be up.
# This script should ideally be run via setup-test-env.sh or manually after setting up DB.

export DATABASE_URL="postgres://test_user:test_password@localhost:5433/test_database"

echo -e "  Pre-compiling backend for integration tests..."
(cd "$PROJECT_ROOT/apiserver" && cargo build) > "$LOG_DIR/prebuild.log" 2>&1 || { echo -e "${RED}Backend build failed. Check $LOG_DIR/prebuild.log${NC}"; exit 1; }

run_parallel_ordered "Phase 3 (Integration & E2E)" \
    "1|apiserver-int|apiserver|timeout 300s cargo test --test '*' -- --test-threads $NUM_WORKERS" \
    "2|playwright-e2e|webdashboard|timeout 300s npx playwright test --workers $NUM_WORKERS"