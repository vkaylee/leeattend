#!/bin/bash
# ==============================================================================
# Test: Verify ai-test.sh prints logs when tests FAIL
#
# Scenarios tested:
#   1. Test runtime failure → marker text appears in output
#   2. "TESTS FAILED" banner appears
#   3. Failure details summary section appears
#   4. Build failure → prebuild error lines appear in output
#   5. Test assertion failure → panic/assertion details appear in output
#
# Strategy:
#   For each scenario, create a fake run-tests.sh that simulates the failure,
#   run ai-test.sh, capture output, and check for expected content.
#
# Usage: ./scripts/test-log-output.sh
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

PASS=0
FAIL=0

pass() { ((PASS++)) || true; echo -e "  ${GREEN}✓${NC} $1"; }
fail() { ((FAIL++)) || true; echo -e "  ${RED}✗${NC} $1"; }

# ── Helper: Run ai-test.sh with a fake run-tests.sh ─────────────────
# Usage: run_fake_test "FAKE_SCRIPT_BODY"
# Returns: captured output in $OUTPUT
run_fake_test() {
    local fake_body="$1"
    # Clean up stale log files from previous runs/scenarios
    rm -f "$PROJECT_ROOT/.test_logs/prebuild.log" 2>/dev/null || true
    rm -f "$PROJECT_ROOT/.test_logs/latest-failure.log" 2>/dev/null || true
    cat > "$REAL_SCRIPT" <<FAKE_EOF
#!/bin/bash
$fake_body
FAKE_EOF
    chmod +x "$REAL_SCRIPT"
    OUTPUT=$(timeout 30 bash "$PROJECT_ROOT/ai-test.sh" --api 2>&1) || true
}

echo -e "${BLUE}${BOLD}🧪 Test: ai-test.sh log output on failure${NC}"

# ── Setup: backup run-tests.sh ──────────────────────────────────────
REAL_SCRIPT="$PROJECT_ROOT/run-tests.sh"
BACKUP_SCRIPT="$PROJECT_ROOT/.run-tests.sh.bak"

if [[ ! -f "$REAL_SCRIPT" ]]; then
    echo -e "${RED}ERROR: run-tests.sh not found${NC}"
    exit 1
fi

cleanup() {
    if [[ -f "$BACKUP_SCRIPT" ]]; then
        mv "$BACKUP_SCRIPT" "$REAL_SCRIPT"
        echo -e "  ${BLUE}Restored original run-tests.sh${NC}"
    fi
    # Clean up any test artifacts
    rm -f "$PROJECT_ROOT/.test_logs/prebuild.log" 2>/dev/null || true
    rm -f "$PROJECT_ROOT/.test_logs/latest-failure.log" 2>/dev/null || true
}
trap cleanup EXIT

cp "$REAL_SCRIPT" "$BACKUP_SCRIPT"

# Skip PTY safety check and log output self-test (prevent recursion)
export _PTY_CHECK_DONE=1
export _LOG_CHECK_DONE=1

# ═══════════════════════════════════════════════════════════════════
# SCENARIO A: Basic test failure (marker text in output)
# ═══════════════════════════════════════════════════════════════════
echo -e "\n  ${BLUE}Scenario A: Basic test failure...${NC}"
MARKER="UNIQUE_TEST_MARKER_$(date +%s)"

run_fake_test "echo '=== FAKE TEST OUTPUT ==='; echo '$MARKER'; echo 'Error: Simulated failure'; exit 1"

# Test 1: Marker appears in output
if echo "$OUTPUT" | grep -q "$MARKER"; then
    pass "Test failure logs are printed to output (marker found)"
else
    fail "Test failure logs are NOT printed (marker '$MARKER' missing)"
    echo -e "  ${RED}Captured output (first 20 lines):${NC}"
    echo "$OUTPUT" | head -20 | sed 's/^/    /'
fi

# Test 2: "TESTS FAILED" banner appears
if echo "$OUTPUT" | grep -q "TESTS FAILED"; then
    pass "TESTS FAILED banner is displayed"
else
    fail "TESTS FAILED banner is missing"
fi

# Test 3: Failure details summary section appears
if echo "$OUTPUT" | grep -qE "(Last 30 lines|Failure Details)"; then
    pass "Failure details summary section is displayed"
else
    fail "Failure details summary section is missing"
fi

# ═══════════════════════════════════════════════════════════════════
# SCENARIO B: Build failure (prebuild.log with compile errors)
# ═══════════════════════════════════════════════════════════════════
echo -e "\n  ${BLUE}Scenario B: Build failure (prebuild.log)...${NC}"

# Ensure .test_logs dir exists
mkdir -p "$PROJECT_ROOT/.test_logs"

# Create fake run-tests.sh that writes a prebuild.log and fails
run_fake_test "
mkdir -p '$PROJECT_ROOT/.test_logs'
cat > '$PROJECT_ROOT/.test_logs/prebuild.log' <<'BUILD_EOF'
   Compiling apiserver v0.1.0 (/workspace/apiserver)
error[E0061]: this function takes 3 arguments but 2 were supplied
   --> src/handlers/leave.rs:42:5
    |
42  |     create_leave_type(pool, body)
    |     ^^^^^^^^^^^^^^^^ --- --- supplied 2 arguments
    |     expected 3 arguments
error: could not compile \`apiserver\` due to 1 previous error
BUILD_EOF
exit 1
"

# Test 4: Build error details appear
if echo "$OUTPUT" | grep -q "Build Error Details"; then
    pass "Build Error Details section is displayed"
else
    fail "Build Error Details section is missing"
    echo -e "  ${RED}Captured output (last 15 lines):${NC}"
    echo "$OUTPUT" | tail -15 | sed 's/^/    /'
fi

# Test 5: Actual error line appears
if echo "$OUTPUT" | grep -q "error\[E0061\]"; then
    pass "Compile error code (E0061) is printed"
else
    fail "Compile error code (E0061) is missing from output"
fi

# Test 6: File location appears
if echo "$OUTPUT" | grep -q "could not compile"; then
    pass "\"could not compile\" message is printed"
else
    fail "\"could not compile\" message is missing"
fi

# ═══════════════════════════════════════════════════════════════════
# SCENARIO C: Test assertion failure (latest-failure.log with panic)
# ═══════════════════════════════════════════════════════════════════
echo -e "\n  ${BLUE}Scenario C: Test assertion failure (latest-failure.log)...${NC}"

# Clean up prebuild.log so it doesn't interfere
rm -f "$PROJECT_ROOT/.test_logs/prebuild.log"

# Create fake run-tests.sh that writes a latest-failure.log and fails
run_fake_test "
mkdir -p '$PROJECT_ROOT/.test_logs'
cat > '$PROJECT_ROOT/.test_logs/latest-failure.log' <<'NEXTEST_EOF'
    Finished \`test\` profile [unoptimized + debuginfo] target(s) in 0.18s
 Nextest run ID abc123
    Starting 204 tests across 2 binaries
        PASS [   0.2s] apiserver::integration auth::session::test_login
        FAIL [   0.1s] apiserver::integration leave::leave_types::test_crud
  stdout ---
    running 1 test
    test leave::leave_types::test_crud ... FAILED
    failures:
        leave::leave_types::test_crud
    test result: FAILED. 0 passed; 1 failed
  stderr ---
    thread 'leave::leave_types::test_crud' panicked at tests/integration/leave/leave_types.rs:75:5:
    assertion \`left == right\` failed: expected CREATED but got NOT_FOUND
      left: 201
     right: 404
     Summary [   4.1s] 113/204 tests run: 112 passed, 1 failed
        FAIL [   0.1s] apiserver::integration leave::leave_types::test_crud
error: test run failed
NEXTEST_EOF
exit 1
"

# Test 7: Failure Details section appears (from latest-failure.log)
if echo "$OUTPUT" | grep -q "Failure Details"; then
    pass "Failure Details section from latest-failure.log is displayed"
else
    fail "Failure Details section is missing"
    echo -e "  ${RED}Captured output (last 15 lines):${NC}"
    echo "$OUTPUT" | tail -15 | sed 's/^/    /'
fi

# Test 8: Panic message appears
if echo "$OUTPUT" | grep -q "panicked"; then
    pass "Panic message is printed"
else
    fail "Panic message is missing from output"
fi

# Test 9: Assertion details appear
if echo "$OUTPUT" | grep -qE "(left:|right:)"; then
    pass "Assertion details (left/right values) are printed"
else
    fail "Assertion details are missing from output"
fi

# Test 10: Test name appears
if echo "$OUTPUT" | grep -q "test_crud"; then
    pass "Failed test name is printed"
else
    fail "Failed test name is missing from output"
fi

# ═══════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════
echo -e "\n${BLUE}${BOLD}═══ Summary ═══${NC}"
echo -e "  ${GREEN}Pass: $PASS${NC}  ${RED}Fail: $FAIL${NC}"

if [[ $FAIL -gt 0 ]]; then
    echo -e "\n${RED}${BOLD}❌ Log output test FAILED${NC}"
    exit 1
else
    echo -e "\n${GREEN}${BOLD}✅ Log output test PASSED${NC}"
    exit 0
fi
