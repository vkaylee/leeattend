#!/bin/bash
# Proxy for Orchestrator Toolbox (Cargo)

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
VENV_PYTHON=$("$SCRIPT_DIR/_ensure-venv.sh")

if [ "$1" = "test" ] || [ "$1" = "nextest" ] || [ "$1" = "llvm-cov" ] || [ "$1" = "tarpaulin" ] || [ "$1" = "grcov" ]; then
    echo "❌ 🤖 [AI AGENT NOTICE] Do NOT use '_cargo.sh $1'!"
    echo "❌ 🤖 [AI AGENT NOTICE] ALWAYS use './test.sh' for testing and coverage."
    echo "❌ 🤖 [AI AGENT NOTICE] './test.sh' handles environment isolation, timeouts, and PTY state properly."
    echo "❌ 🤖 [AI AGENT NOTICE] Command aborted. Please update your tool call and try again."
    exit 1
fi

# Pass all arguments to orchestrator run command
$VENV_PYTHON "$SCRIPT_DIR/_orchestrator.py" run cargo "$@"

