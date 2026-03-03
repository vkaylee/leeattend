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

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="/tmp/ai-test-$(date +%s).txt"

echo "🤖 AI Test Runner — PTY-safe wrapper for run-tests.sh"
echo "📝 Logging to: $LOG_FILE"
echo "🚀 Running: ./run-tests.sh $*"
echo "================================================================"

# Launch run-tests.sh in a new session to prevent PTY FD inheritance
setsid bash -c "cd '$SCRIPT_DIR' && ./run-tests.sh $* > '$LOG_FILE' 2>&1" \
    </dev/null >/dev/null 2>&1 &

BG_PID=$!

# Poll until the setsid process finishes
POLL_INTERVAL=5
ELAPSED=0
while kill -0 "$BG_PID" 2>/dev/null; do
    sleep $POLL_INTERVAL
    ELAPSED=$((ELAPSED + POLL_INTERVAL))

    # Show the last line as a progress indicator every 15s
    if (( ELAPSED % 15 == 0 )); then
        LAST_LINE=$(tail -1 "$LOG_FILE" 2>/dev/null || echo "...")
        echo "  ⏳ ${ELAPSED}s — $LAST_LINE"
    fi
done

# Wait for full exit and capture exit code
wait "$BG_PID" 2>/dev/null
EXIT_CODE=$?

echo ""
echo "================================================================"
echo "📋 FULL OUTPUT:"
echo "================================================================"
cat "$LOG_FILE"

# Clean up temp file
rm -f "$LOG_FILE"

exit $EXIT_CODE
