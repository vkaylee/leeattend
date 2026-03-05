#!/bin/bash
# ==============================================================================
# Test: Verify ai-test.sh prints logs when tests FAIL
#
# Strategy:
#   1. Create a fake run-tests.sh that outputs known text and exits non-zero
#   2. Run ai-test.sh (which calls run-tests.sh)
#   3. Capture stdout+stderr and check that the known text appears
#   4. Restore original run-tests.sh
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

echo -e "${BLUE}${BOLD}🧪 Test: ai-test.sh log output on failure${NC}"

# ── Setup: backup run-tests.sh and replace with a fake ──────────────
REAL_SCRIPT="$PROJECT_ROOT/run-tests.sh"
BACKUP_SCRIPT="$PROJECT_ROOT/.run-tests.sh.bak"

if [[ ! -f "$REAL_SCRIPT" ]]; then
    echo -e "${RED}ERROR: run-tests.sh not found${NC}"
    exit 1
fi

cleanup() {
    # Restore original run-tests.sh
    if [[ -f "$BACKUP_SCRIPT" ]]; then
        mv "$BACKUP_SCRIPT" "$REAL_SCRIPT"
        echo -e "  ${BLUE}Restored original run-tests.sh${NC}"
    fi
}
trap cleanup EXIT

cp "$REAL_SCRIPT" "$BACKUP_SCRIPT"

# Create fake run-tests.sh that outputs a known marker and fails
MARKER="UNIQUE_TEST_MARKER_$(date +%s)"
cat > "$REAL_SCRIPT" << FAKE_EOF
#!/bin/bash
echo "=== FAKE TEST OUTPUT ==="
echo "$MARKER"
echo "Error: Simulated test failure for verification"
echo "Details: This text MUST appear in ai-test.sh output"
exit 1
FAKE_EOF
chmod +x "$REAL_SCRIPT"

# ── Test 1: Logs appear in stdout when test fails ────────────────────
echo -e "\n  ${BLUE}Running ai-test.sh with fake failing test...${NC}"

# Skip PTY safety check and log output self-test (prevent recursion)
export _PTY_CHECK_DONE=1
export _LOG_CHECK_DONE=1

OUTPUT=$(timeout 30 bash "$PROJECT_ROOT/ai-test.sh" --api 2>&1) || true

echo -e "  ${BLUE}Checking output for marker text...${NC}"

if echo "$OUTPUT" | grep -q "$MARKER"; then
    pass "Test failure logs are printed to output (marker found)"
else
    fail "Test failure logs are NOT printed (marker '$MARKER' missing)"
    echo -e "  ${RED}Captured output (first 20 lines):${NC}"
    echo "$OUTPUT" | head -20 | sed 's/^/    /'
fi

# ── Test 2: "TESTS FAILED" banner appears ────────────────────────────
if echo "$OUTPUT" | grep -q "TESTS FAILED"; then
    pass "TESTS FAILED banner is displayed"
else
    fail "TESTS FAILED banner is missing"
fi

# ── Test 3: "Last 30 lines" summary section appears ──────────────────
if echo "$OUTPUT" | grep -q "Last 30 lines"; then
    pass "Last 30 lines summary section is displayed"
else
    fail "Last 30 lines summary section is missing"
fi

# ── Summary ──────────────────────────────────────────────────────────
echo -e "\n${BLUE}${BOLD}═══ Summary ═══${NC}"
echo -e "  ${GREEN}Pass: $PASS${NC}  ${RED}Fail: $FAIL${NC}"

if [[ $FAIL -gt 0 ]]; then
    echo -e "\n${RED}${BOLD}❌ Log output test FAILED${NC}"
    exit 1
else
    echo -e "\n${GREEN}${BOLD}✅ Log output test PASSED${NC}"
    exit 0
fi
