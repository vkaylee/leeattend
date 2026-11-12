#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
source "$ROOT/scripts/playwright-timeout.sh"
playwright_timeout_init

SUPERVISED=false
if [[ "${1:-}" == --supervised ]]; then
  SUPERVISED=true
  shift
fi

ROLE="${PLAYWRIGHT_ROLE:-employee}"
if [[ "${1:-}" != --* && -n "${1:-}" ]]; then
  ROLE="$1"
  shift
fi
case "$ROLE" in
  employee|team-lead|shift-manager|hr-manager|admin) ;;
  *)
    printf 'Unsupported Playwright role: %s\n' "$ROLE" >&2
    exit 2
    ;;
esac

if [[ "${1:-}" != --endpoint || -z "${2:-}" ]]; then
  printf '%s\n' 'Usage: playwright-mcp-role.sh ROLE --endpoint ENDPOINT [MCP_ARGS...]' >&2
  exit 2
fi
ENDPOINT="$2"
shift 2

case "$ROLE" in
  employee) ROLE_GUIDANCE="Access only pages and actions permitted to an employee." ;;
  team-lead) ROLE_GUIDANCE="Access only pages and actions permitted to a team lead." ;;
  shift-manager) ROLE_GUIDANCE="Access only pages and actions permitted to a shift manager." ;;
  hr-manager) ROLE_GUIDANCE="Access only pages and actions permitted to an HR manager." ;;
  admin) ROLE_GUIDANCE="Access only pages and actions permitted to an administrator." ;;
esac
ROLE_INSTRUCTIONS="Use Playwright MCP only for UI tasks. This browser session starts unauthenticated; open ${ENDPOINT}/login before authentication. ${ROLE_GUIDANCE} Never bypass application authorization or inspect secrets, cookies, or storage tokens."

if [[ "$SUPERVISED" == false ]]; then
  PYTHON="$ROOT/.venv/bin/python3"
  if [[ ! -x "$PYTHON" ]]; then
    printf '%s\n' 'Project Python environment missing; run scripts/_python.sh --version before starting Playwright MCP.' >&2
    exit 1
  fi
  exec "$PYTHON" "$ROOT/scripts/playwright-idle-watchdog.py" \
    --idle-seconds "${PLAYWRIGHT_MCP_IDLE_TIMEOUT_SECS:-10}" \
    --initialize-instructions "$ROLE_INSTRUCTIONS" \
    -- bash "$ROOT/scripts/playwright-mcp-role.sh" --supervised "$ROLE" --endpoint "$ENDPOINT" "$@"
fi
if [[ -n "${DOCKER_COMPOSE_CMD:-}" ]]; then
  read -r -a COMPOSE <<<"${DOCKER_COMPOSE_CMD}"
elif command -v podman-compose >/dev/null 2>&1; then
  COMPOSE=(podman-compose)
elif command -v docker >/dev/null 2>&1; then
  COMPOSE=(docker compose)
else
  printf '%s\n' 'No supported Compose engine found.' >&2
  exit 1
fi

RUN_ARGS=(run --rm --no-deps -T)

MCP_COMMAND=(
  --headless
  --isolated
  --no-sandbox
  --browser=chrome
  --caps=vision,pdf,devtools
  --console-level=info
  --snapshot-boxes
  "--allowed-origins=$ENDPOINT"
  --timeout-action=5000
  --timeout-navigation=30000
  --timeout-settle=1000
  --idle-timeout=0
  "$@"
)


PLAYWRIGHT_COMPOSE=("${COMPOSE[@]}" -p leedevkit-dev -f "$ROOT/.compose/docker-compose.dev.yml" --profile mcp)
playwright_build_image_if_missing "localhost/leedevkit-dev_playwright-mcp:latest" "${PLAYWRIGHT_COMPOSE[@]}"
exec "${PLAYWRIGHT_COMPOSE[@]}" \
  "${RUN_ARGS[@]}" playwright-mcp \
  "${MCP_COMMAND[@]}"
