#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"

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

"${COMPOSE[@]}" \
  -p leedevkit-dev \
  -f "$ROOT/.compose/docker-compose.dev.yml" \
  --profile mcp \
  build playwright-mcp >&2

exec "${COMPOSE[@]}" \
  -p leedevkit-dev \
  -f "$ROOT/.compose/docker-compose.dev.yml" \
  --profile mcp \
  run --rm --no-deps -T playwright-mcp "$@"
