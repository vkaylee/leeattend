#!/bin/bash

# ── Podman Compose wrappers ─────────────────────────────────────────
#
# dc()          — Normal use. Redirects stdin only (</dev/null).
# dc_detached() — Cleanup/background use. Creates new session (setsid)
#                 and full FD isolation. 60s hard timeout prevents
#                 indefinite PTY hang when podman/conmon misbehaves.
#
dc() {
    $DOCKER_COMPOSE_CMD "$@" </dev/null
}
dc_detached() {
    local _timeout="${DOCKER_COMPOSE_TIMEOUT:-60}"
    setsid -w $DOCKER_COMPOSE_CMD "$@" </dev/null >/dev/null 2>&1 &
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


