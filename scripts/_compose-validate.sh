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
if [[ -n "${POSTGRES_IMAGE:-}" && -n "${PGBOUNCER_IMAGE:-}" \
  && -n "${POSTGRES_TLS_DIR:-}" && -n "${PGBOUNCER_TLS_DIR:-}" \
  && -n "${LICENSE_FILE:-}" ]]; then
  "${COMPOSE[@]}" -f docker-compose.yml config >/dev/null
else
  printf '%s\n' 'Production Compose validation skipped: standalone deployment variables are not configured.' >&2
fi
"${COMPOSE[@]}" -f .compose/docker-compose.dev.yml config >/dev/null
PROJECT_ROOT="$ROOT" "${COMPOSE[@]}" \
  --profile infra-db --profile infra-redis --profile int-go --profile web \
  -f .compose/docker-compose.test.yml config >/dev/null
printf '%s\n' 'Compose configuration valid: production, development, test'
