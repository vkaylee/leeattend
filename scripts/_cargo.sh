#!/bin/bash
# Proxy for Orchestrator Toolbox (Cargo)

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
VENV_PYTHON=$("$SCRIPT_DIR/_ensure-venv.sh")

if [ "$1" = "test" ]; then
    echo "❌ 🤖 [AI AGENT NOTICE] Do NOT use './scripts/_cargo.sh test'!"
    echo "❌ 🤖 [AI AGENT NOTICE] ALWAYS use './scripts/_cargo.sh nextest run' for faster test execution."
    echo "❌ 🤖 [AI AGENT NOTICE] Command aborted. Please update your tool call and try again."
    exit 1
fi

# Pass all arguments to orchestrator run command
$VENV_PYTHON "$SCRIPT_DIR/_orchestrator.py" run cargo "$@"

