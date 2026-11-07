#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
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
  if ! "${COMPOSE[@]}" \
    -p leedevkit-dev \
    -f "$ROOT/.compose/docker-compose.dev.yml" \
    --profile mcp \
    run --rm --no-deps -T \
    -v "$AUTH_STATE:/run/secrets/playwright-auth.json:ro" \
    -e PLAYWRIGHT_AUTH_STATE=/run/secrets/playwright-auth.json \
    -e PLAYWRIGHT_BASE_URL=http://caddy \
    --entrypoint node playwright-mcp \
    /opt/playwright-mcp/check-auth.cjs >/dev/null 2>&1; then
    "$ROOT/scripts/playwright-auth-role.sh" "$ROLE"
  fi
else
  "$ROOT/scripts/playwright-auth-role.sh" "$ROLE"
fi

RUN_ARGS=(run --rm --no-deps -T)
MCP_ARGS=()
if [[ -f "$AUTH_STATE" ]]; then
  RUN_ARGS+=(
    -v "$AUTH_STATE:/run/secrets/playwright-auth.json:ro"
  )
  MCP_ARGS+=(--storage-state /run/secrets/playwright-auth.json)
fi

exec "${COMPOSE[@]}" \
  -p leedevkit-dev \
  -f "$ROOT/.compose/docker-compose.dev.yml" \
  --profile mcp \
  "${RUN_ARGS[@]}" playwright-mcp \
  "${MCP_ARGS[@]}" "$@"
