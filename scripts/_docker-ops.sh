#!/bin/bash

# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e "\033[0;31m❌ SECURITY VIOLATION: Direct execution of this module is forbidden.\033[0m"
    echo -e "👉 This script is an internal module. Use: \033[1m./test.sh\033[0m"
    exit 1
fi


# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e " [0;31m❌ SECURITY VIOLATION: Direct execution of this module is forbidden. [0m"
    echo -e "👉 This script is an internal module. Use:  [1m./test.sh [0m"
    exit 1
fi


# ── Podman Compose wrappers ─────────────────────────────────────────
#
# dc()          — Normal use. Redirects stdin only (</dev/null).
# dc_detached() — Cleanup/background use. Creates new session (setsid)
#                 and full FD isolation. 60s hard timeout prevents
#                 indefinite PTY hang when podman/conmon misbehaves.
#
dc() {
    # shellcheck disable=SC2086
    $TOOLBOX_COMPOSE_CMD "$@" </dev/null
}
dc_detached() {
    local _timeout="${DOCKER_COMPOSE_TIMEOUT:-60}"
    # shellcheck disable=SC2086
    setsid -w $TOOLBOX_COMPOSE_CMD "$@" </dev/null >/dev/null 2>&1 &
    local _pid=$!
    (sleep "$_timeout"; kill -KILL -$_pid 2>/dev/null) </dev/null &
    local _watchdog=$!
    wait $_pid 2>/dev/null
    local _rc=$?
    kill $_watchdog 2>/dev/null
    wait $_watchdog 2>/dev/null
    return $_rc
}
export -f dc dc_detached


