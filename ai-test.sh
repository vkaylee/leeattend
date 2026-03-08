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

# ── Process Tracking ─────────────────────────────────────────────────
# Track all spawned child PIDs so we can kill them on exit.
# This prevents orphan processes (setsid children, Docker daemons)
# from holding terminal/PTY file descriptors open indefinitely.
_CHILD_PIDS=()

cleanup_children() {
    if [[ ${#_CHILD_PIDS[@]} -gt 0 ]]; then
        for pid in "${_CHILD_PIDS[@]}"; do
            # Kill the entire process group rooted at the setsid leader
            kill -- -"$pid" 2>/dev/null || true
            kill "$pid" 2>/dev/null || true
        done
        # Brief wait then force-kill any survivors
        sleep 0.5
        for pid in "${_CHILD_PIDS[@]}"; do
            kill -9 -- -"$pid" 2>/dev/null || true
            kill -9 "$pid" 2>/dev/null || true
        done
    fi
}
trap cleanup_children EXIT INT TERM

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
_CHILD_PIDS+=($!)

# FIX: Ensure log file exists before polling (setsid may not have written yet)
touch "$LOG_FILE"
# Stream log output in real-time with tail -f
# This ensures logs appear immediately, especially on failures.
tail -f "$LOG_FILE" </dev/null &
_TAIL_PID=$!
_CHILD_PIDS+=($_TAIL_PID)

# Wait for the exit code file to appear (= subprocess finished)
while [ ! -f "$EXIT_CODE_FILE" ]; do
    sleep 1
done

# Read exit code from file
EXIT_CODE=$(cat "$EXIT_CODE_FILE" 2>/dev/null || echo "1")

# Give tail -f time to flush remaining output.
# On failure, on_task_failure writes the full log AFTER setting exit code,
# so we must wait longer for that output to arrive in the log file.
if [ "$EXIT_CODE" -eq 0 ]; then
    sleep 0.5
else
    sleep 2
fi

# Kill tail -f (no longer needed, process is done)
kill $_TAIL_PID 2>/dev/null || true
wait $_TAIL_PID 2>/dev/null || true

echo ""
echo "================================================================"
if [ "$EXIT_CODE" -eq 0 ]; then
    echo "✅ ALL TESTS PASSED"
else
    echo "❌ TESTS FAILED (exit code: $EXIT_CODE)"
    echo "================================================================"
    echo ""
    # Print failure details from the nextest/test log (clean output, no ANSI).
    # on_task_failure creates a symlink at .test_logs/latest-failure.log
    FAILURE_LOG="$SCRIPT_DIR/.test_logs/latest-failure.log"
    PREBUILD_LOG="$SCRIPT_DIR/.test_logs/prebuild.log"
    if [ -f "$FAILURE_LOG" ]; then
        echo "──── Failure Details (from $FAILURE_LOG) ────"
        # Show FAIL lines, panic messages, and assertion details
        grep -E '(FAIL|panicked|assertion|left:|right:|error:|stderr)' "$FAILURE_LOG" 2>/dev/null | head -20
        echo ""
        echo "──── Full Failure Context ────"
        # Show everything from first FAIL line onward (compact)
        grep -n 'FAIL' "$FAILURE_LOG" 2>/dev/null | head -1 | while IFS=: read -r line_num _rest; do
            sed -n "${line_num},\$p" "$FAILURE_LOG" | head -40
        done
    elif [ -f "$PREBUILD_LOG" ]; then
        echo "──── Build Error Details ────"
        grep -E '(^error|^\s+-->|could not compile)' "$PREBUILD_LOG" 2>/dev/null | tail -20
    else
        echo "──── Last 30 lines ────"
        tail -30 "$LOG_FILE" 2>/dev/null || true
    fi
fi
echo "================================================================"

# Clean up temp files
rm -f "$LOG_FILE" "$EXIT_CODE_FILE"

# cleanup_children runs via EXIT trap — kills all tracked processes
exit "$EXIT_CODE"
