#!/bin/bash

# ── Hard timeout for cleanup operations (seconds) ────────────────────
CLEANUP_DEADLINE="${CLEANUP_DEADLINE:-120}"

# ── /proc FD sweep safety: max PIDs to scan before bail-out ─────────
FD_SWEEP_MAX_PROCS="${FD_SWEEP_MAX_PROCS:-500}"

# Cleanup handler: stop containers on exit
cleanup() {
    local exit_code=$?

    # ── PTY FD Isolation ─────────────────────────────────────────────
    # CRITICAL: Podman/podman-compose processes spawned below inherit
    # the script's PTY file descriptors. Even after the script exits
    # and the shell returns to prompt, these orphan FDs keep the PTY
    # "open" — causing AI agent tools (command_status) to report the
    # command as still RUNNING with "No output".
    #
    # Fix: redirect stdout/stderr to a log file BEFORE spawning any
    # podman processes. This severs the PTY FD chain so cleanup
    # processes cannot hold the parent terminal open.
    local _cleanup_log="${LOG_DIR:-/tmp}/cleanup.log"
    echo -e "\n🧹 Cleaning up test environment..." >&2
    exec 1>>"$_cleanup_log" 2>&1
    exec 0</dev/null

    # ── Cleanup deadline watchdog ────────────────────────────────────
    # If cleanup itself hangs (e.g. podman-compose down stuck on
    # network), this timer force-kills the entire process group.
    # Uses a background subshell with all FDs severed from PTY.
    (
        sleep "$CLEANUP_DEADLINE"
        echo "CLEANUP_DEADLINE: force-killing process group after ${CLEANUP_DEADLINE}s" >> "$_cleanup_log"
        kill -KILL -$$ 2>/dev/null || true
    ) </dev/null >/dev/null 2>&1 &
    local _deadline_pid=$!
    # Arm RETURN trap to cancel the deadline on normal cleanup exit
    trap 'kill $_deadline_pid 2>/dev/null; wait $_deadline_pid 2>/dev/null' RETURN

    # ── Kill all child processes ─────────────────────────────────────
    # Ignore TERM in this handler so we survive our own cleanup kills.
    trap '' TERM
    pkill -TERM -P $$ 2>/dev/null || true
    sleep 0.5
    pkill -KILL -P $$ 2>/dev/null || true

    # Also kill any cargo/nextest processes left inside containers
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        for ctr in apiserver_test agent_test; do
            timeout 5 podman exec "${COMPOSE_PROJECT_NAME}-${ctr}-1" \
                pkill -f 'cargo|nextest' </dev/null >/dev/null 2>&1 || true
        done
    fi

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # Stop containers with short timeout, then KILL
        dc_detached down --remove-orphans --timeout 5 || true
        # Force kill any remaining containers for this project
        podman ps -q --filter "label=com.docker.compose.project=$COMPOSE_PROJECT_NAME" | xargs -r podman kill >/dev/null 2>&1 || true
        
        # Remove ephemeral volumes
        podman volume rm "leeattend-test_shared-data" \
            </dev/null >/dev/null 2>&1 || true
        echo "✅ Environment cleaned up (cache preserved)."
    fi

    # ── Final Status Reporting ───────────────────────────────────────
    local _status_file="${PROJECT_ROOT:-.}/.test_logs/cleanup.status.json"
    mkdir -p "$(dirname "$_status_file")" 2>/dev/null || true
    if [[ "$exit_code" -ne 0 ]]; then
        echo "{\"status\": \"failed\", \"exit_code\": $exit_code, \"log\": \"$_cleanup_log\"}" > "$_status_file"
    else
        echo "{\"status\": \"success\", \"exit_code\": $exit_code, \"log\": \"$_cleanup_log\"}" > "$_status_file"
    fi

    # ── Layer 3a: Process group kill (catch grandchildren) ──────────
    # `kill -$$` sends signal to the entire process group, including
    # grandchildren that `pkill -P $$` misses.
    kill -TERM -$$ 2>/dev/null || true
    sleep 0.3
    kill -KILL -$$ 2>/dev/null || true

    # ── Layer 3b: /proc FD sweep (catch reparented orphans) ──────────
    # Find processes still attached to our PTY that escaped the process
    # group kill (e.g. reparented to PID 1). Verify they actually hold
    # a PTY FD via /proc before killing — avoids false positives.
    _our_tty=$(ps -o tty= -p $$ 2>/dev/null | tr -d ' ')
    if [[ -n "$_our_tty" && "$_our_tty" != "?" ]]; then
        # Use pipe instead of process substitution to avoid spawning a
        # subshell that inherits PTY fds (< <(cmd) creates a subshell).
        ps -t "$_our_tty" -o pid= 2>/dev/null | while IFS= read -r _pid; do
            _pid=$(echo "$_pid" | tr -d ' ')
            [[ -z "$_pid" || "$_pid" == "$$" || "$_pid" == "$PPID" || "$_pid" == "1" ]] && continue
            # Only kill if process actually holds a PTY file descriptor
            if ls -la /proc/"$_pid"/fd 2>/dev/null | grep -q "/dev/pts/"; then
                kill -KILL "$_pid" 2>/dev/null || true
            fi
        done
    fi

    # ── Final sweep ──────────────────────────────────────────────────
    pkill -KILL -P $$ 2>/dev/null || true

    # Cancel deadline (RETURN trap also handles this)
    kill $_deadline_pid 2>/dev/null || true
    wait $_deadline_pid 2>/dev/null || true

    exit "$exit_code"
}


# --- Watchdog (Safety Net) ---
# Spawns a fully detached background process that kills the parent
# after a timeout. All FDs are severed from PTY so the watchdog
# itself cannot be the cause of a PTY hang.
start_watchdog() {
    local timeout_sec=$1
    (
        sleep "$timeout_sec"
        echo -e "\n\n🚨 WATCHDOG: GLOBAL TIMEOUT REACHED (${timeout_sec}s)! Force-killing main script..." >&2
        kill -KILL -$$ 2>/dev/null || true
        sleep 2
        kill -9 $$ 2>/dev/null || true
    ) </dev/null >/dev/null 2>&1 &
    WATCHDOG_PID=$!
    export WATCHDOG_PID
}
