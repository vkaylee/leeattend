#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"
source "$ROOT/scripts/playwright-timeout.sh"
playwright_timeout_init

ROLE="${1:-employee}"
shift || true
case "$ROLE" in
  employee) CREDENTIAL_PREFIX=EMPLOYEE;;
  team-lead) CREDENTIAL_PREFIX=TEAM_LEAD;;
  shift-manager) CREDENTIAL_PREFIX=SHIFT_MANAGER;;
  hr-manager) CREDENTIAL_PREFIX=HR_MANAGER;;
  admin) CREDENTIAL_PREFIX=ADMIN;;
  *) printf 'Unsupported Playwright role: %s\n' "$ROLE" >&2; exit 2;;
esac

CREDENTIALS_FILE="${PLAYWRIGHT_CREDENTIALS_FILE:-$ROOT/.secrets/playwright-credentials.env}"
AUTH_DIR="${PLAYWRIGHT_AUTH_DIR:-$ROOT/.secrets/playwright-auth}"
AUTH_STATE="$AUTH_DIR/$ROLE.json"

if [[ ! -f "$CREDENTIALS_FILE" ]]; then
  printf 'Missing credentials file: %s\n' "$CREDENTIALS_FILE" >&2
  exit 1
fi

set -a
# shellcheck disable=SC1090
source "$CREDENTIALS_FILE"
set +a
EMAIL_VAR="PLAYWRIGHT_${CREDENTIAL_PREFIX}_EMAIL"
PASSWORD_VAR="PLAYWRIGHT_${CREDENTIAL_PREFIX}_PASSWORD"
EMAIL="${!EMAIL_VAR:-}"
PASSWORD="${!PASSWORD_VAR:-}"
: "${EMAIL:?Missing $EMAIL_VAR in credentials file}"
: "${PASSWORD:?Missing $PASSWORD_VAR in credentials file}"

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

mkdir -p "$AUTH_DIR"
chmod 700 "$AUTH_DIR"
LOGIN_FILE="$(mktemp "$AUTH_DIR/.login.XXXXXX.env")"
trap 'rm -f "$LOGIN_FILE"' EXIT
chmod 600 "$LOGIN_FILE"
printf 'PLAYWRIGHT_LOGIN_EMAIL=%s\nPLAYWRIGHT_LOGIN_PASSWORD=%s\n' "$EMAIL" "$PASSWORD" > "$LOGIN_FILE"

COMPOSE_PROJECT="${PLAYWRIGHT_COMPOSE_PROJECT:-leedevkit-dev}"
COMPOSE_FILE="${PLAYWRIGHT_COMPOSE_FILE:-$ROOT/.compose/docker-compose.dev.yml}"
PLAYWRIGHT_COMPOSE=("${COMPOSE[@]}" -p "$COMPOSE_PROJECT" -f "$COMPOSE_FILE" --profile mcp)
playwright_build_image_if_missing "localhost/leedevkit-dev_playwright-mcp:latest" "${PLAYWRIGHT_COMPOSE[@]}"
playwright_timeout_run "${COMPOSE[@]}" -p leedevkit-dev -f "$ROOT/.compose/docker-compose.dev.yml" --profile mcp run --rm --no-deps -T \
  -v "$AUTH_DIR:/run/secrets:rw" \
  -e PLAYWRIGHT_LOGIN_FILE=/run/secrets/$(basename "$LOGIN_FILE") \
  -e PLAYWRIGHT_BASE_URL=http://caddy \
  -e PLAYWRIGHT_AUTH_STATE=/run/secrets/$ROLE.json \
  --entrypoint node playwright-mcp /opt/playwright-mcp/prepare-auth.cjs </dev/null >&2

chmod 600 "$AUTH_STATE"
printf 'Playwright auth state saved for role %s.\n' "$ROLE" >&2
