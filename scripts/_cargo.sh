#!/bin/bash
# Proxy for Orchestrator Toolbox (Cargo)

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
VENV_PYTHON=$("$SCRIPT_DIR/_ensure-venv.sh")

# Detect test-related subcommands (either directly or prefixed with "cargo")
IS_TEST_CMD=false
TEST_CMD=""
ARG_START_INDEX=2

if [ "$1" = "test" ] || [ "$1" = "nextest" ] || [ "$1" = "llvm-cov" ] || [ "$1" = "tarpaulin" ] || [ "$1" = "grcov" ]; then
    IS_TEST_CMD=true
    TEST_CMD="$1"
    ARG_START_INDEX=2
elif [ "$1" = "cargo" ] && { [ "$2" = "test" ] || [ "$2" = "nextest" ] || [ "$2" = "llvm-cov" ] || [ "$2" = "tarpaulin" ] || [ "$2" = "grcov" ]; }; then
    IS_TEST_CMD=true
    TEST_CMD="$2"
    ARG_START_INDEX=3
fi

if [ "$IS_TEST_CMD" = true ]; then
    SUGGESTED_CMD="./test.sh api"
    if [ "$TEST_CMD" = "llvm-cov" ] || [ "$TEST_CMD" = "tarpaulin" ] || [ "$TEST_CMD" = "grcov" ]; then
        SUGGESTED_CMD="./test.sh api --coverage"
    fi
    
    # Append any extra arguments
    if [ $# -ge $ARG_START_INDEX ]; then
        SUGGESTED_CMD="$SUGGESTED_CMD ${*:ARG_START_INDEX}"
    fi

    echo "========================================================================="
    echo "❌ 🤖 [AI AGENT NOTICE] Do NOT use '_cargo.sh $TEST_CMD'!"
    echo "❌ 🤖 [AI AGENT NOTICE] ALWAYS use './test.sh' for testing and coverage."
    echo "❌ 🤖 [AI AGENT NOTICE] './test.sh' handles environment isolation, timeouts, and PTY state properly."
    echo ""
    echo "👉 Please run this equivalent command instead:"
    echo "   $SUGGESTED_CMD"
    echo "========================================================================="
    exit 1
fi

# Pass all arguments to orchestrator run command
$VENV_PYTHON "$SCRIPT_DIR/_orchestrator.py" run cargo "$@"

