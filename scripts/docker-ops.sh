#!/bin/bash

# ── Podman Compose wrappers ─────────────────────────────────────────
#
# dc()          — Normal use. Redirects stdin only (</dev/null).
# dc_detached() — Cleanup use. Redirect all FDs (Podman is daemonless,
#                 so no setsid needed — children die with parent).
#
dc() {
    $DOCKER_COMPOSE_CMD "$@" </dev/null
}
dc_detached() {
    $DOCKER_COMPOSE_CMD "$@" </dev/null >/dev/null 2>&1
}
export -f dc dc_detached


