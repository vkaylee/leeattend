#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
source "$ROOT/scripts/playwright-timeout.sh"
playwright_timeout_init

ROLE="${1:-employee}"
shift || true
case "$ROLE" in
  employee|team-lead|shift-manager|hr-manager|admin) ;;
  *)
    printf 'Unsupported Playwright role: %s\n' "$ROLE" >&2
    exit 2
    ;;
esac

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

AUTH_DIR="${PLAYWRIGHT_AUTH_DIR:-$ROOT/.secrets/playwright-auth}"
AUTH_STATE="$AUTH_DIR/$ROLE.json"

if [[ -f "$AUTH_STATE" ]]; then
  if ! playwright_timeout_run "${COMPOSE[@]}" \
    -p leedevkit-dev \
    -f "$ROOT/.compose/docker-compose.dev.yml" \
    --profile mcp \
    run --rm --no-deps -T \
    -v "$AUTH_STATE:/run/secrets/playwright-auth.json:ro" \
    -e PLAYWRIGHT_AUTH_STATE=/run/secrets/playwright-auth.json \
    -e PLAYWRIGHT_BASE_URL=http://caddy \
    --entrypoint node playwright-mcp \
    /opt/playwright-mcp/check-auth.cjs </dev/null >/dev/null 2>&1; then
    "$ROOT/scripts/playwright-auth-role.sh" "$ROLE" </dev/null
  fi
else
  "$ROOT/scripts/playwright-auth-role.sh" "$ROLE" </dev/null
fi

RUN_ARGS=(run --rm --no-deps -T)
if [[ -f "$AUTH_STATE" ]]; then
  RUN_ARGS+=(
    -v "$AUTH_STATE:/run/secrets/playwright-auth.json:ro"
    -e PLAYWRIGHT_MCP_STORAGE_STATE=/run/secrets/playwright-auth.json
  )
fi

MCP_COMMAND=(
  --headless
  --isolated
  --no-sandbox
  --browser=chrome
  --caps=vision,pdf,devtools
  --console-level=info
  --snapshot-boxes
  --allowed-origins=http://caddy
  --timeout-action=5000
  --timeout-navigation=30000
  --timeout-settle=1000
  "$@"
)


playwright_idle_run "$ROOT/scripts/_python.sh" "$ROOT/scripts/playwright-idle-watchdog.py" \
  "${COMPOSE[@]}" \
  -p leedevkit-dev \
  -f "$ROOT/.compose/docker-compose.dev.yml" \
  --profile mcp \
  "${RUN_ARGS[@]}" playwright-mcp \
  "${MCP_COMMAND[@]}"
