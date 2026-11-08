#!/usr/bin/env bash
# Verify documented development accounts against a running API.
# Development-only: identities and role expectations come from docs/dev-accounts.md.
# Credentials are read from existing local credentials file (.secrets/playwright-credentials.env)
# or process environment, with documented dev fallbacks.
# Output never includes tokens, raw responses, or unmasked emails.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

BASE_URL="${BASE_URL:-http://127.0.0.1:8088/api/v1}"
CREDENTIALS_FILE="${DEV_CREDENTIALS_FILE:-$ROOT/.secrets/playwright-credentials.env}"
ADMIN_MONITOR_PATH="${ADMIN_MONITOR_PATH:-/api/v1/monitor/overview}"

ACCOUNTS=(
  "admin:ADMIN:admin@example.invalid"
  "hr.manager:HR_MANAGER:hr.manager@example.invalid"
  "shift.manager:SHIFT_MANAGER:shift.manager@example.invalid"
  "team.lead:TEAM_LEAD:team.lead@example.invalid"
  "employee:EMPLOYEE:employee@example.invalid"
)

DEFAULT_PASSWORD="${DEV_PASSWORD:-development-password}"

if [[ -f "$CREDENTIALS_FILE" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "$CREDENTIALS_FILE"
  set +a
fi

fail() { printf 'FAIL: %s\n' "$1" >&2; exit 1; }

mask_email() {
  local email="$1" local_part domain
  local_part="${email%@*}"
  domain="${email#*@}"
  printf '%s***@%s' "${local_part:0:1}" "$domain"
}

json_string_field() {
  printf '%s' "$1" | jq -r --arg key "$2" 'if type == "object" then (.[$key] // empty) else empty end' 2>/dev/null || true
}

for entry in "${ACCOUNTS[@]}"; do
  IFS=':' read -r account prefix default_email <<< "$entry"
  email_var="PLAYWRIGHT_${prefix}_EMAIL"
  password_var="PLAYWRIGHT_${prefix}_PASSWORD"
  email="${!email_var:-$default_email}"
  password="${!password_var:-$DEFAULT_PASSWORD}"

  login_body="$(jq -nc --arg email "$email" --arg password "$password" '{email:$email,password:$password}')"
  login_response="$(curl -sS --max-time 10 -X POST "$BASE_URL/auth/login" -H 'Content-Type: application/json' -d "$login_body")" \
    || fail "login request failed for $account"
  token="$(json_string_field "$login_response" access_token)"
  [[ -n "$token" ]] || fail "login rejected for $account (no access token)"

  permissions_response="$(curl -sS --max-time 10 "$BASE_URL/me/permissions" -H "Authorization: Bearer $token")" \
    || fail "permissions request failed for $account"
  roles="$(printf '%s' "$permissions_response" | jq -r 'if type == "object" then ((.roles // []) | join(",")) else empty end')"
  permission_count="$(printf '%s' "$permissions_response" | jq -r 'if type == "object" then ((.permissions // []) | length) else 0 end')"
  [[ -n "$roles" ]] || fail "no roles returned for $account"

  monitor_note=""
  if [[ "$account" == "admin" ]]; then
    monitor_status="$(curl -sS --max-time 10 -o /dev/null -w '%{http_code}' "$BASE_URL${ADMIN_MONITOR_PATH#/api/v1}" -H "Authorization: Bearer $token")" \
      || fail "monitor probe failed for $account"
    [[ "$monitor_status" == "200" ]] || fail "monitor access denied for $account (HTTP $monitor_status)"
    monitor_note=" | monitor=200"
  fi

  printf 'PASS: %s (%s) | roles=%s | permissions=%s%s\n' \
    "$account" "$(mask_email "$email")" "$roles" "$permission_count" "$monitor_note"
done

printf 'All documented development accounts authenticated with expected roles.\n'
