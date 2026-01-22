#!/bin/bash
set -euo pipefail
SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SOURCE_DIR/test-utils.sh"

# Note: Integration tests require the DB environment to be up.
# This script should ideally be run via setup-test-env.sh or manually after setting up DB.

export DATABASE_URL="postgres://test_user:test_password@localhost:5433/leeattend_test_template"

echo -e "  Pre-compiling backend for integration tests..."
(cd "$PROJECT_ROOT/apiserver" && cargo test --no-run -j ${CARGO_BUILD_JOBS:-2}) > "$LOG_DIR/prebuild.log" 2>&1 || { echo -e "${RED}Backend build failed. Check $LOG_DIR/prebuild.log${NC}"; exit 1; }

# Tối ưu hóa: Yêu cầu Rust test sử dụng database có sẵn nếu TestDatabase hỗ trợ
# Chúng ta sẽ chạy cargo test với ít thread hơn để tránh lock tranh chấp quá mạnh
export RUST_TEST_THREADS=$NUM_WORKERS

run_parallel_ordered "Phase 3 (Integration & E2E)" \
    "1|apiserver-int|apiserver|timeout 120s cargo test --test '*' -- --test-threads $NUM_WORKERS" \
    "2|playwright-e2e|webdashboard|timeout 120s npx playwright test --workers $NUM_WORKERS"