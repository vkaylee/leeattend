#!/bin/bash
# Proxy for Orchestrator Toolbox (NPM/Bun)

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
VENV_PYTHON=$("$SCRIPT_DIR/_ensure-venv.sh")

IS_TEST_CMD=false
SUGGESTED_CMD="./test.sh web"

if [ "$1" = "test" ]; then
    IS_TEST_CMD=true
    SUGGESTED_CMD="./test.sh web"
    if [ $# -gt 1 ]; then
        SUGGESTED_CMD="$SUGGESTED_CMD ${*:2}"
    fi
elif [ "$1" = "vitest" ] || [ "$1" = "playwright" ]; then
    IS_TEST_CMD=true
    if [ "$1" = "vitest" ]; then
        SUGGESTED_CMD="./test.sh web --unit-only"
    else
        SUGGESTED_CMD="./test.sh web --e2e-only"
    fi
    if [ $# -gt 1 ]; then
        SUGGESTED_CMD="$SUGGESTED_CMD ${*:2}"
    fi
elif [ "$1" = "run" ]; then
    if [ "$2" = "vitest" ] || [ "$2" = "test:unit" ]; then
        IS_TEST_CMD=true
        SUGGESTED_CMD="./test.sh web --unit-only"
        if [ $# -gt 2 ]; then
            SUGGESTED_CMD="$SUGGESTED_CMD ${*:3}"
        fi
    elif [ "$2" = "playwright" ] || [ "$2" = "test:e2e" ]; then
        IS_TEST_CMD=true
        SUGGESTED_CMD="./test.sh web --e2e-only"
        if [ $# -gt 2 ]; then
            SUGGESTED_CMD="$SUGGESTED_CMD ${*:3}"
        fi
    elif [[ "$2" =~ ^test ]] || [[ "$2" =~ ^cov ]]; then
        IS_TEST_CMD=true
        SUGGESTED_CMD="./test.sh web"
        if [ $# -gt 2 ]; then
            SUGGESTED_CMD="$SUGGESTED_CMD ${*:3}"
        fi
    fi
fi

if [ "$IS_TEST_CMD" = true ]; then
    echo "========================================================================="
    echo "❌ 🤖 [AI AGENT NOTICE] Do NOT use '_npm.sh' for testing!"
    echo "❌ 🤖 [AI AGENT NOTICE] ALWAYS use './test.sh' for testing and coverage."
    echo "❌ 🤖 [AI AGENT NOTICE] './test.sh' handles environment isolation, timeouts, and PTY state properly."
    echo ""
    echo "👉 Please run this equivalent command instead:"
    echo "   $SUGGESTED_CMD"
    echo "========================================================================="
    exit 1
fi

# Pass all arguments to orchestrator run command
$VENV_PYTHON "$SCRIPT_DIR/_orchestrator.py" run npm "$@"
