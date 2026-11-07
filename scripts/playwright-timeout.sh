#!/usr/bin/env bash

playwright_timeout_init() {
  local runtime="${PLAYWRIGHT_MCP_MAX_RUNTIME:-120}"
  local grace="${PLAYWRIGHT_MCP_KILL_GRACE:-15}"

  if ! command -v timeout >/dev/null 2>&1; then
    printf '%s\n' 'GNU timeout is required for Playwright process cleanup.' >&2
    return 1
  fi
  if [[ ! "$runtime" =~ ^[1-9][0-9]*$ || "$runtime" -gt 3600 ]]; then
    printf '%s\n' 'PLAYWRIGHT_MCP_MAX_RUNTIME must be an integer from 1 to 3600 seconds.' >&2
    return 2
  fi
  if [[ ! "$grace" =~ ^[1-9][0-9]*$ || "$grace" -gt 300 ]]; then
    printf '%s\n' 'PLAYWRIGHT_MCP_KILL_GRACE must be an integer from 1 to 300 seconds.' >&2
    return 2
  fi

  PLAYWRIGHT_MCP_MAX_RUNTIME="$runtime"
  PLAYWRIGHT_MCP_KILL_GRACE="$grace"
}

playwright_timeout_run() {
  local status=0
  timeout --foreground \
    --signal=TERM \
    --kill-after="${PLAYWRIGHT_MCP_KILL_GRACE}s" \
    "${PLAYWRIGHT_MCP_MAX_RUNTIME}s" "$@" || status=$?

  if [[ "$status" -eq 124 || "$status" -eq 137 ]]; then
    printf 'Playwright process timed out after %ss; termination grace was %ss.\n' \
      "$PLAYWRIGHT_MCP_MAX_RUNTIME" "$PLAYWRIGHT_MCP_KILL_GRACE" >&2
  fi
  return "$status"
}
playwright_idle_run() {
  local idle="${PLAYWRIGHT_MCP_IDLE_TIMEOUT_SECS:-60}"
  local python_launcher="$1"
  local watchdog="$2"
  shift 2

  if [[ ! "$idle" =~ ^[1-9][0-9]*$ || "$idle" -gt 60 ]]; then
    printf '%s\n' 'PLAYWRIGHT_MCP_IDLE_TIMEOUT_SECS must be an integer from 1 to 60 seconds.' >&2
    return 2
  fi

  "$python_launcher" "$watchdog" \
    --idle-seconds "$idle" \
    -- "$@"
}
