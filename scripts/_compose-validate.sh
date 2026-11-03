#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
if command -v podman-compose >/dev/null 2>&1; then
  COMPOSE=(podman-compose)
elif command -v docker-compose >/dev/null 2>&1; then
  COMPOSE=(docker-compose)
elif command -v docker >/dev/null 2>&1; then
  COMPOSE=(docker compose)
else
  echo "No supported Compose engine found" >&2
  exit 1
fi

cd "$ROOT"
"${COMPOSE[@]}" -f docker-compose.yml config >/dev/null
"${COMPOSE[@]}" -f docker-compose.yml -f .compose/docker-compose.dev.yml config >/dev/null
PROJECT_ROOT="$ROOT" "${COMPOSE[@]}" -f .compose/docker-compose.test.yml config >/dev/null
"${COMPOSE[@]}" -f docker-compose.standalone.yml config >/dev/null
printf '%s\n' 'Compose configuration valid: production, development, test, standalone'
