#!/usr/bin/env bash
# Managed by LeeDevKit — hermetic container execution wrapper
set -euo pipefail
if [[ "${1:-}" == "bun" || "${1:-}" == "npm" || "${1:-}" == "npx" || "${1:-}" == "bunx" ]]; then
    exec leedevkit exec webdashboard -- "$@"
fi
exec leedevkit exec webdashboard -- bun "$@"
