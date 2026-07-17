#!/bin/bash
# Proxy for Orchestrator Toolbox (Diesel)

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
VENV_PYTHON=$("$SCRIPT_DIR/_ensure-venv.sh")

# The orchestrator relocated into the per-project .leedevkit install
# (gitignored). Prefer the legacy in-tree path for forward-compat, then
# fall back to .leedevkit/scripts/.
ORCHESTRATOR="$SCRIPT_DIR/_orchestrator.py"
if [ ! -f "$ORCHESTRATOR" ]; then
    ORCHESTRATOR="$SCRIPT_DIR/../.leedevkit/scripts/_orchestrator.py"
fi

# Pass all arguments to orchestrator run command
$VENV_PYTHON "$ORCHESTRATOR" run diesel "$@"
