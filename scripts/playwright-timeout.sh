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
  if [[ -z "${PLAYWRIGHT_MCP_MAX_RUNTIME:-}" ]]; then
    playwright_timeout_init
  fi
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

playwright_build_image_if_missing() {
  local image="$1"
  shift

  if [[ "${COMPOSE[0]}" == podman-compose || "${COMPOSE[0]}" == podman ]]; then
    podman image exists "$image" && return 0
  else
    docker image inspect "$image" >/dev/null 2>&1 && return 0
  fi

  playwright_timeout_run "$@" build playwright-mcp </dev/null >&2
}
