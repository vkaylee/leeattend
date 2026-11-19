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
VALIDATION_ENV_FILE="${COMPOSE_VALIDATION_ENV_FILE:-.env.standalone.example}"
if [[ ! -f "$VALIDATION_ENV_FILE" ]]; then
  printf 'Production Compose validation env file missing: %s\n' "$VALIDATION_ENV_FILE" >&2
  exit 1
fi
"${COMPOSE[@]}" --env-file "$VALIDATION_ENV_FILE" -f docker-compose.yml config >/dev/null
"${COMPOSE[@]}" -f .compose/docker-compose.dev.yml config >/dev/null
PROJECT_ROOT="$ROOT" "${COMPOSE[@]}" \
  --profile infra-db --profile infra-redis --profile int-go --profile web \
  -f .compose/docker-compose.test.yml config >/dev/null
PROJECT_ROOT="$ROOT" "${COMPOSE[@]}" \
  --profile infra-db --profile infra-redis --profile int-go --profile web \
  -f .compose/docker-compose.test.yml -f .compose/docker-compose.ci.yml config >/dev/null
printf '%s\n' 'Compose configuration valid: production, development, test, CI'
