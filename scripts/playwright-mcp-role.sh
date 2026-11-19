#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
source "$ROOT/scripts/playwright-timeout.sh"
playwright_timeout_init

SUPERVISED=false
BUILD=false
while [[ "${1:-}" == --supervised || "${1:-}" == --build ]]; do
  case "$1" in
    --supervised) SUPERVISED=true ;;
    --build) BUILD=true ;;
  esac
  shift
done

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
  printf '%s\n' 'Usage: playwright-mcp-role.sh [--build] ROLE --endpoint ENDPOINT [MCP_ARGS...]' >&2
  exit 2
fi
ENDPOINT="$2"
shift 2

ENDPOINT_HOST="${ENDPOINT#*://}"
ENDPOINT_HOST="${ENDPOINT_HOST%%/*}"
if [[ -z "$ENDPOINT_HOST" ]]; then
  printf 'Invalid Playwright endpoint: %s\n' "$ENDPOINT" >&2
  exit 2
fi

COMPOSE_PROJECT="${PLAYWRIGHT_COMPOSE_PROJECT:-leedevkit-dev}"
COMPOSE_FILE="${PLAYWRIGHT_COMPOSE_FILE:-$ROOT/.compose/docker-compose.dev.yml}"
if [[ ! -f "$COMPOSE_FILE" ]]; then
  printf 'Missing Playwright Compose file: %s\n' "$COMPOSE_FILE" >&2
  exit 1
fi

case "$ROLE" in
  employee) ROLE_GUIDANCE="Access only pages and actions permitted to an employee." ;;
  team-lead) ROLE_GUIDANCE="Access only pages and actions permitted to a team lead." ;;
  shift-manager) ROLE_GUIDANCE="Access only pages and actions permitted to a shift manager." ;;
  hr-manager) ROLE_GUIDANCE="Access only pages and actions permitted to an HR manager." ;;
  admin) ROLE_GUIDANCE="Access only pages and actions permitted to an administrator." ;;
esac
VIEWPORT_MANIFEST="$ROOT/config/ui-viewports.json"
if [[ ! -f "$VIEWPORT_MANIFEST" ]]; then
  printf 'Missing viewport manifest: %s\n' "$VIEWPORT_MANIFEST" >&2
  exit 1
fi
VIEWPORT_GUIDANCE="$($ROOT/scripts/_python.sh - "$VIEWPORT_MANIFEST" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as manifest_file:
    manifest = json.load(manifest_file)

baseline = next(
    viewport for viewport in manifest["supported"]
    if viewport["name"] == manifest["baseline"]
)
by_name = {viewport["name"]: viewport for viewport in manifest["supported"]}
format_tier = lambda names: ", ".join(
    f'{name}={by_name[name]["width"]}x{by_name[name]["height"]}' for name in names
)
print(
    f'Viewport policy: desktop-first. Default to {baseline["name"]} '
    f'({baseline["width"]}x{baseline["height"]}). Required desktop: '
    f'{format_tier(manifest["tiers"]["required"])}. Secondary tablet: '
    f'{format_tier(manifest["tiers"]["secondary"])}. Mobile smoke only: '
    f'{format_tier(manifest["tiers"]["smoke"])}. Before each UI scenario, '
    'announce selected viewport, call browser_resize, then verify '
    'window.innerWidth and window.innerHeight.'
)
PY
)"
ROLE_INSTRUCTIONS="Use Playwright MCP only for UI tasks. This browser session starts unauthenticated; open ${ENDPOINT}/login before authentication. ${ROLE_GUIDANCE} Never bypass application authorization or inspect secrets, cookies, or storage tokens. ${VIEWPORT_GUIDANCE}"

if [[ "$SUPERVISED" == false && "$BUILD" == true ]]; then
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
  PLAYWRIGHT_COMPOSE=("${COMPOSE[@]}" -p "$COMPOSE_PROJECT" -f "$COMPOSE_FILE" --profile mcp)
  playwright_timeout_run "${PLAYWRIGHT_COMPOSE[@]}" build playwright-mcp </dev/null >&2
fi

if [[ "$SUPERVISED" == false ]]; then
  PYTHON="$ROOT/.venv/bin/python3"
  if [[ ! -x "$PYTHON" ]]; then
    printf '%s\n' 'Project Python environment missing; run scripts/_python.sh --version before starting Playwright MCP.' >&2
    exit 1
  fi
  exec "$PYTHON" "$ROOT/scripts/playwright-idle-watchdog.py" \
    --idle-seconds "${PLAYWRIGHT_MCP_IDLE_TIMEOUT_SECS:-10}" \
    --session-idle-seconds "${PLAYWRIGHT_MCP_SESSION_IDLE_TIMEOUT_SECS:-300}" \
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
  "--allowed-hosts=$ENDPOINT_HOST"
  "--allowed-origins=$ENDPOINT"
  --timeout-action=5000
  --timeout-navigation=30000
  --timeout-settle=1000
  --idle-timeout=0
  "$@"
)


PLAYWRIGHT_COMPOSE=("${COMPOSE[@]}" -p "$COMPOSE_PROJECT" -f "$COMPOSE_FILE" --profile mcp)
IMAGE="localhost/leedevkit-dev_playwright-mcp:latest"
if [[ "$BUILD" == true ]]; then
  playwright_timeout_run "${PLAYWRIGHT_COMPOSE[@]}" build playwright-mcp </dev/null >&2
elif [[ "${COMPOSE[0]}" == podman-compose || "${COMPOSE[0]}" == podman ]]; then
  podman image exists "$IMAGE" || {
    printf 'Playwright MCP image missing: %s\n' "$IMAGE" >&2
    printf 'Build it with: %s --build %s --endpoint %s\n' "$0" "$ROLE" "$ENDPOINT" >&2
    exit 1
  }
else
  docker image inspect "$IMAGE" >/dev/null 2>&1 || {
    printf 'Playwright MCP image missing: %s\n' "$IMAGE" >&2
    printf 'Build it with: %s --build %s --endpoint %s\n' "$0" "$ROLE" "$ENDPOINT" >&2
    exit 1
  }
fi
exec "${PLAYWRIGHT_COMPOSE[@]}" \
  "${RUN_ARGS[@]}" playwright-mcp \
  "${MCP_COMMAND[@]}"
