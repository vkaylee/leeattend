#!/bin/bash
# ==============================================================================
# PTY-Safe Test Runner for AI Agents
# Wraps run-tests.sh with setsid to prevent PTY file descriptor inheritance
# that causes Docker commands to appear hung in AI agent terminals.
#
# Usage: ./ai-test.sh [any run-tests.sh arguments]
# Example: ./ai-test.sh --api --smoke
#          ./ai-test.sh --web --smoke
#          ./ai-test.sh --api --test=my_test_name
#          ./ai-test.sh --all
# ==============================================================================

# Handle --help explicitly (exit 0) and no-args fallback (exit 1)
if [[ $# -eq 0 ]]; then
    echo "❌ Error: No arguments provided." >&2
    echo ""
    echo "Usage: ./ai-test.sh [any run-tests.sh arguments]"
    echo "Run './ai-test.sh --help' for full usage."
    exit 1
fi

for _arg in "$@"; do
    if [[ "$_arg" == "--help" || "$_arg" == "-h" ]]; then
        # Delegate to run-tests.sh --help directly (no setsid needed)
        exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/run-tests.sh" --help
    fi
done

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── PTY Safety Gate ──────────────────────────────────────────────────
# Run STATIC scan only before anything else. Runtime tests (which use
# the `script` command) are excluded here because `script` creates PTY
# child processes that themselves leak FDs in git hook context.
# For full runtime tests, run: ./scripts/test-pty-safety.sh
if [[ -z "${_PTY_CHECK_DONE:-}" && -f "$SCRIPT_DIR/scripts/test-pty-safety.sh" ]]; then
    echo "🔒 Running PTY safety check (static)..."
    if ! "$SCRIPT_DIR/scripts/test-pty-safety.sh" --static; then
        echo "❌ PTY safety check failed — fix issues above before running tests."
        exit 1
    fi
    echo ""
fi

# ── Log Output Self-Test Gate ────────────────────────────────────────
# Verify that ai-test.sh correctly prints logs on test failure.
# Uses _LOG_CHECK_DONE guard to prevent recursion (same pattern as PTY gate).
if [[ -z "${_LOG_CHECK_DONE:-}" && -f "$SCRIPT_DIR/scripts/test-log-output.sh" ]]; then
    echo "🧪 Running log output self-test..."
    if ! "$SCRIPT_DIR/scripts/test-log-output.sh"; then
        echo "❌ Log output self-test failed — fix issues above before running tests."
        exit 1
    fi
    echo ""
fi

# FIX: Use a single timestamp+PID to avoid race between two $(date) calls
_TS="$(date +%s)-$$"
LOG_FILE="/tmp/ai-test-${_TS}.log"
EXIT_CODE_FILE="/tmp/ai-test-exit-${_TS}.txt"

echo "🤖 AI Test Runner — PTY-safe wrapper for run-tests.sh"
echo "📝 Logging to: $LOG_FILE"
echo "🚀 Running: ./run-tests.sh $*"
echo "================================================================"

# FIX: Build properly quoted argument string for bash -c
# $* inside bash -c "..." expands unquoted → word-splitting + injection
QUOTED_ARGS=""
for arg in "$@"; do
    QUOTED_ARGS="$QUOTED_ARGS '$(echo "$arg" | sed "s/'/'\\\\''/g")'"
done

# Launch run-tests.sh in a new session, writing exit code to a file
# This fully detaches from the PTY — no fd inheritance possible
setsid bash -c "cd '$SCRIPT_DIR' && ./run-tests.sh $QUOTED_ARGS > '$LOG_FILE' 2>&1; echo \$? > '$EXIT_CODE_FILE'" \
    </dev/null >/dev/null 2>&1 &

# FIX: Ensure log file exists before polling (setsid may not have written yet)
touch "$LOG_FILE"

# Poll until the exit code file appears (= subprocess finished)
POLL_INTERVAL=5
ELAPSED=0
while [ ! -f "$EXIT_CODE_FILE" ]; do
    sleep $POLL_INTERVAL
    ELAPSED=$((ELAPSED + POLL_INTERVAL))

    # Show the last line as a progress indicator every 15s
    if (( ELAPSED % 15 == 0 )); then
        LAST_LINE=$(tail -1 "$LOG_FILE" 2>/dev/null || echo "...")
        echo "  ⏳ ${ELAPSED}s — $LAST_LINE"
    fi
done

# Read exit code from file
EXIT_CODE=$(cat "$EXIT_CODE_FILE" 2>/dev/null || echo "1")

echo ""
echo "================================================================"
if [ "$EXIT_CODE" -eq 0 ]; then
    echo "✅ ALL TESTS PASSED"
    echo "================================================================"
    cat "$LOG_FILE" || true
else
    echo "❌ TESTS FAILED (exit code: $EXIT_CODE)"
    echo "================================================================"
    # FIX: Output to stdout (not stderr) so AI agent tools capture it.
    # Previous >&2 was invisible to most agent terminal readers.
    cat "$LOG_FILE" || true
    echo ""
    echo "──── Last 30 lines ────"
    tail -30 "$LOG_FILE" 2>/dev/null || true
fi

# Clean up temp files
rm -f "$LOG_FILE" "$EXIT_CODE_FILE"

exit "$EXIT_CODE"
