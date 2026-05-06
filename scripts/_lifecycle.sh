#!/bin/bash

# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e "\033[0;31m❌ SECURITY VIOLATION: Direct execution of this module is forbidden.\033[0m"
    echo -e "👉 This script is an internal module. Use: \033[1m./test.sh\033[0m"
    exit 1
fi


# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e "\033[0;31m❌ SECURITY VIOLATION: Direct execution of this module is forbidden.\033[0m"
    echo -e "👉 This script is an internal module. Use: \033[1m./test.sh\033[0m"
    exit 1
fi


# ── Hard timeout for cleanup operations (seconds) ────────────────────
CLEANUP_DEADLINE="${CLEANUP_DEADLINE:-120}"

# ── /proc FD sweep safety: max PIDs to scan before bail-out ─────────
FD_SWEEP_MAX_PROCS="${FD_SWEEP_MAX_PROCS:-500}"

# --- Lifecycle Management ---

lifecycle_up() {
    local mode="${MODE:-all}"
    local profiles=()

    # Translate Mode into logical profile sets (Task + Infrastructure)
    case "$mode" in
        api)
            # Logic Tasks
            profiles=("--profile" "lint-api" "--profile" "unit-api")
            # Required Infrastructure for API Logic (DB is mandatory for sqlx/lint)
            profiles+=("--profile" "infra-db" "--profile" "infra-db-init")
            ;;
        integration)
            # Full Backend Suite
            profiles=(
                "--profile" "lint-api" "--profile" "unit-api" "--profile" "int-api"
                "--profile" "infra-db" "--profile" "infra-db-init" "--profile" "infra-redis"
            )
            ;;
        web)
            # Frontend Logic Only (Lite by default)
            profiles=("--profile" "lint-web" "--profile" "unit-web" "--profile" "e2e-web")
            ;;
        all)
            # Everything
            profiles=(
                "--profile" "lint-api" "--profile" "unit-api" "--profile" "int-api"
                "--profile" "lint-web" "--profile" "unit-web" "--profile" "e2e-web"
                "--profile" "infra-db" "--profile" "infra-db-init" "--profile" "infra-redis"
            )
            ;;
    esac

    # Ensure a clean slate first for the requested profiles only
    echo -e "${BLUE}🧹 Cleaning up stale containers...${NC}"
    timeout 15 bash -c "$DOCKER_COMPOSE_CMD ${profiles[*]} down --remove-orphans --timeout 1" </dev/null >/dev/null 2>&1 || true

    echo -e "${BLUE}🚀 Bringing up test infrastructure [Mode: $mode]...${NC}"
    dc "${profiles[@]}" up -d

    # Wait for DB if backend is involved - Using Container Health Status
    if [[ "$mode" != "web" ]]; then
        echo -n "  Waiting for backend services to be healthy..."
        for _ in {1..60}; do
            # Check if apiserver is healthy (it depends on db and redis health)
            if [[ "$("$CONTAINER_ENGINE" inspect --format='{{.State.Health.Status}}' "leeattend-test_apiserver_1" 2>/dev/null)" == "healthy" ]]; then
                echo -e " ${GREEN}Ready!${NC}"
                return 0
            fi
            echo -n "."
            sleep 1
        done
        echo -e "\n${RED}❌ Infrastructure failed to reach healthy state!${NC}"
        "$CONTAINER_ENGINE" ps -a --filter "name=leeattend-test"
        return 1
    else
        # Web mode: wait for bun_init to finish installing dependencies
        echo -n "  Waiting for web dependencies (bun install)..."
        for _ in {1..120}; do
            if [[ "$("$CONTAINER_ENGINE" inspect --format='{{.State.Health.Status}}' "leeattend-test_bun_init_1" 2>/dev/null)" == "healthy" ]]; then
                echo -e " ${GREEN}Ready!${NC}"
                return 0
            fi
            echo -n "."
            sleep 1
        done
        echo -e "\n${RED}❌ Web dependencies failed to install!${NC}"
        "$CONTAINER_ENGINE" ps -a --filter "name=leeattend-test"
        return 1
    fi
}

lifecycle_down() {
    local mode="${MODE:-all}"
    local profiles=()

    case "$mode" in
        api|integration) profiles=("--profile" "backend") ;;
        web) profiles=("--profile" "frontend") ;;
        all) profiles=("--profile" "backend" "--profile" "frontend") ;;
    esac

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        echo -e "${BLUE}🧹 Lifecycle: Tearing down infrastructure [Mode: $mode]...${NC}"
        # Stop containers with short timeout, then KILL
        dc "${profiles[@]}" down --remove-orphans --timeout 5 >/dev/null 2>&1 || true
        
        # Force kill AND remove any remaining containers for this project
        local _ids
        _ids=$("$CONTAINER_ENGINE" ps -aq --filter "label=com.docker.compose.project=$COMPOSE_PROJECT_NAME")
        if [[ -n "$_ids" ]]; then
            echo "$_ids" | xargs -r "$CONTAINER_ENGINE" rm -f >/dev/null 2>&1 || true
        fi
        
        # Remove ephemeral volumes
        "$CONTAINER_ENGINE" volume rm "${COMPOSE_PROJECT_NAME}_shared-data" \
            </dev/null >/dev/null 2>&1 || true
        echo -e "${GREEN}✅ Infrastructure cleaned up.${NC}"
    fi
}

# Cleanup handler: stop processes and call lifecycle_down on exit
cleanup() {
    local exit_code=$?

    # ── PTY FD Isolation ─────────────────────────────────────────────
    local _cleanup_log="${LOG_DIR:-/tmp}/cleanup.log"
    echo -e "\n🧹 Cleaning up test environment..." >&2
    exec 1>>"$_cleanup_log" 2>&1
    exec 0</dev/null

    # ── Cleanup deadline watchdog ────────────────────────────────────
    (
        sleep "$CLEANUP_DEADLINE"
        echo "CLEANUP_DEADLINE: force-killing process group after ${CLEANUP_DEADLINE}s" >> "$_cleanup_log"
        kill -KILL -$$ 2>/dev/null || true
    ) </dev/null >/dev/null 2>&1 &
    local _deadline_pid=$!
    trap 'kill $_deadline_pid 2>/dev/null; wait $_deadline_pid 2>/dev/null' RETURN

    # ── Kill all child processes ─────────────────────────────────────
    trap '' TERM
    pkill -TERM -P $$ 2>/dev/null || true
    sleep 0.5
    pkill -KILL -P $$ 2>/dev/null || true

    # Also kill any cargo/nextest processes left inside containers
    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        for ctr in apiserver agent-main; do
            timeout 5 "$CONTAINER_ENGINE" exec "${COMPOSE_PROJECT_NAME}-${ctr}-1" \
                pkill -f 'cargo|nextest' </dev/null >/dev/null 2>&1 || true
        done
    fi

    # ── Tear down infrastructure ─────────────────────────────────────
    lifecycle_down

    # ── Final Status Reporting ───────────────────────────────────────
    local _status_file="${PROJECT_ROOT:-.}/.test_logs/cleanup.status.json"
    mkdir -p "$(dirname "$_status_file")" 2>/dev/null || true
    if [[ "$exit_code" -ne 0 ]]; then
        echo "{\"status\": \"failed\", \"exit_code\": $exit_code, \"log\": \"$_cleanup_log\"}" > "$_status_file"
    else
        echo "{\"status\": \"success\", \"exit_code\": $exit_code, \"log\": \"$_cleanup_log\"}" > "$_status_file"
    fi

    # ── Layer 3a: Process group kill (catch grandchildren) ──────────
    kill -TERM -$$ 2>/dev/null || true
    sleep 0.3
    kill -KILL -$$ 2>/dev/null || true

    # ── Layer 3b: /proc FD sweep (catch reparented orphans) ──────────
    _our_tty=$(ps -o tty= -p $$ 2>/dev/null | tr -d ' ')
    if [[ -n "$_our_tty" && "$_our_tty" != "?" ]]; then
        ps -t "$_our_tty" -o pid= 2>/dev/null | while IFS= read -r _pid; do
            _pid=$(echo "$_pid" | tr -d ' ')
            [[ -z "$_pid" || "$_pid" == "$$" || "$_pid" == "$PPID" || "$_pid" == "1" ]] && continue
            # shellcheck disable=SC2010
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
