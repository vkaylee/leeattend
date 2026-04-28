#!/bin/bash

show_help() {
    echo -e "${BLUE}${BOLD}🚀 LeeAttend Test Runner${NC}"
    echo "================================================================================"
    echo "Single entry point for ALL test and container operations in LeeAttend."
    echo "Never run scripts in scripts/ directly — they are internal modules."
    echo ""
    echo -e "${BOLD}📋 USAGE${NC}"
    echo "    ./run-tests.sh [MODE] [OPTIONS]"
    echo ""
    echo -e "${BOLD}🎯 TEST MODES (choose one):${NC}"
    echo -e "    ${GREEN}--all${NC}              Run ALL tests: Lint → Unit → Integration → E2E"
    echo -e "    ${GREEN}--api${NC}              Run API tests: apiserver (Rust) + agent-main (Rust)"
    echo -e "    ${GREEN}--web${NC}              Run Web tests: webdashboard (React + Playwright)"
    echo -e "    ${GREEN}--integration${NC}      Run ONLY integration + E2E tests (skip lint/unit)"
    echo ""
    echo -e "${BOLD}📊 COVERAGE MODE (combine with test modes):${NC}"
    echo -e "    ${GREEN}--coverage${NC}         Measure test coverage (Rust: llvm-cov, Web: vitest)"
    echo -e "                       Outputs: .test_logs/coverage-*.lcov + HTML reports"
    echo ""
    echo -e "${BOLD}🐳 CONTAINER ENVIRONMENT MODE:${NC}"
    echo -e "    ${GREEN}--docker=ENV CMD${NC}   Run any podman-compose command for ENV (dev|test|prod)"
    echo -e "                       ENV profiles: dev → docker-compose.dev.yml"
    echo -e "                                      test → docker-compose.test.yml"
    echo -e "                                      prod → docker-compose.yml only"
    echo -e "                       CMD = any podman-compose subcommand (up, down, build, exec, logs, ps...)"
    echo ""
    echo -e "${BOLD}⚡ TEST OPTIONS (combine with test modes):${NC}"
    echo -e "    ${GREEN}--test=PATTERN${NC}     Run ONLY tests matching PATTERN (regex for Rust, string for Playwright)"
    echo -e "    ${GREEN}--test-only${NC}        Skip lint + unit phases, run ONLY integration/E2E matching PATTERN"
    echo -e "    ${GREEN}--component=NAME${NC}   Limit to one component: apiserver | agent-main | webdashboard"
    echo -e "    ${GREEN}--smoke${NC}            Quick check: run lint + unit + fast smoke-tagged integration"
    echo -e "    ${GREEN}--list${NC}             List available tests without running them"
    echo -e "    ${GREEN}--dry-run${NC}          Show what commands would execute, without running anything"
    echo -e "    ${GREEN}--shard=N/M${NC}        CI parallelization: run shard N of M (e.g., --shard=1/3)"
    echo -e "    ${GREEN}--timeout=SECONDS${NC}  Override global timeout (default: 1800 = 30 min)"
    echo ""
    echo -e "${BOLD}🗄️ DATABASE MANAGEMENT:${NC}"
    echo -e "    ${GREEN}--db-backup${NC}        Backup PostgreSQL (dev) → ./backups/leeattend_YYYYMMDD_HHMMSS.dump"
    echo -e "    ${GREEN}--db-restore=FILE${NC}  Restore PostgreSQL (dev) from a .dump or .sql file"
    echo ""
    echo -e "${BOLD}🔧 CACHE MANAGEMENT:${NC}"
    echo -e "    ${GREEN}--warm-cache${NC}       Pre-compile all dependencies (run once on fresh machine / CI)"
    echo -e "    ${GREEN}--cache-stats${NC}      Show size and status of Cargo/NPM caches"
    echo -e "    ${GREEN}--no-cache${NC}         Force fresh build, ignoring all caches"
    echo -e "    ${GREEN}--clean-cache${NC}      Delete cache only (keep containers + volumes running)"
    echo -e "    ${GREEN}--clean${NC}            Full reset: stop containers, delete volumes + cache"
    echo ""
    echo -e "${BOLD}📜 LOG ACCESS:${NC}"
    echo -e "    ${GREEN}--logs${NC}             Dump all test logs from the persistent volume"
    echo -e "    ${GREEN}--tail-logs${NC}        Live-tail test logs from the volume"
    echo -e "    ${GREEN}--dev-logs${NC}         Dump service logs (apiserver, agent-main, webdashboard)"
    echo -e "    ${GREEN}--dev-logs=follow${NC}  Live-tail service logs"
    echo ""
    echo -e "${BOLD}════════════════════════════════════════ EXAMPLES ════════════════════════════════════════${NC}"
    echo ""
    echo -e "${YELLOW}# 1. Running a specific test by name:${NC}"
    echo "    ./run-tests.sh --api --test=my_test_name"
    echo "    ./run-tests.sh --api --test=auth::test_login_success"
    echo "    # Skip lint if it has compile errors (fast iteration):"
    echo "    ./run-tests.sh --api --test-only --test=my_test_name"
    echo ""
    echo -e "${YELLOW}# 2. Running tests for a specific component only:${NC}"
    echo "    ./run-tests.sh --api --component=apiserver"
    echo "    ./run-tests.sh --api --component=agent-main"
    echo "    ./run-tests.sh --web --component=webdashboard"
    echo "    ./run-tests.sh --api --component=apiserver --test=my_test"
    echo ""
    echo -e "${YELLOW}# 3. Debugging a failing test (step by step):${NC}"
    echo "    ./run-tests.sh --api --dry-run                          # Step 1: see what would run"
    echo "    ./run-tests.sh --api --test-only --test=failing_test    # Step 2: run only that test"
    echo "    ./run-tests.sh --logs                                    # Step 3: read logs if it still fails"
    echo ""
    echo -e "${YELLOW}# 4. Quick smoke check before committing:${NC}"
    echo "    ./run-tests.sh --api --smoke    # lint + unit + fast integration (smoke-tagged)"
    echo "    ./run-tests.sh --all --smoke    # all components, smoke integration only"
    echo ""
    echo -e "${YELLOW}# 5. Full test pipeline:${NC}"
    echo "    ./run-tests.sh --all            # Lint → Unit → Integration → E2E"
    echo "    ./run-tests.sh --api            # API only: Lint → Unit → Integration"
    echo "    ./run-tests.sh --web            # Web only: Lint → Unit → Playwright E2E"
    echo ""
    echo -e "${YELLOW}# 6. Container environment management:${NC}"
    echo "    ./run-tests.sh --docker=dev up -d                # Start dev environment"
    echo "    ./run-tests.sh --docker=dev down                 # Stop dev environment"
    echo "    ./run-tests.sh --docker=dev logs -f              # Tail all dev logs"
    echo "    ./run-tests.sh --docker=dev logs -f apiserver    # Tail one service log"
    echo "    ./run-tests.sh --docker=dev exec apiserver bash  # Shell into container"
    echo "    ./run-tests.sh --docker=dev build apiserver      # Rebuild one service"
    echo "    ./run-tests.sh --docker=test build               # Rebuild test environment"
    echo "    ./run-tests.sh --docker=prod ps                  # Check prod service status"
    echo ""
    echo -e "${YELLOW}# 7. Database backup & restore:${NC}"
    echo "    ./run-tests.sh --db-backup                                # Create backup"
    echo "    ./run-tests.sh --db-restore=./backups/leeattend_*.dump   # Restore from file"
    echo ""
    echo -e "${YELLOW}# 8. First time setup / fresh machine:${NC}"
    echo "    ./run-tests.sh --all --warm-cache    # Pre-compile all deps first"
    echo "    ./run-tests.sh --all                 # Then run tests (much faster)"
    echo ""
    echo -e "${YELLOW}# 9. CI/CD parallel sharding:${NC}"
    echo "    ./run-tests.sh --api --shard=1/3     # Job 1 of 3"
    echo "    ./run-tests.sh --api --shard=2/3     # Job 2 of 3"
    echo "    ./run-tests.sh --api --shard=3/3     # Job 3 of 3"
    echo ""
    echo -e "${YELLOW}# 10. Troubleshooting & maintenance:${NC}"

    echo "    ./run-tests.sh --cache-stats         # Check cache size"
    echo "    ./run-tests.sh --clean-cache         # Clear cache, keep containers"
    echo "    ./run-tests.sh --clean               # Full reset"
    echo "    ./run-tests.sh --logs                # Read test logs from Docker volume"
    echo "    ./run-tests.sh --tail-logs           # Live-tail logs"
    echo ""
    echo -e "${BOLD}🔄 TEST PATTERN FORMATS:${NC}"
    echo -e "${YELLOW}  API - Rust (cargo nextest):${NC}"
    echo "    test_name          → any test whose name contains 'test_name' (regex)"
    echo "    module::test_name  → exact test in a module"
    echo "    module             → all tests in a module"
    echo ""
    echo -e "${YELLOW}  Web - Playwright:${NC}"
    echo "    login_page         → tests whose title contains 'login_page'"
    echo "    @smoke             → tests tagged with @smoke"
    echo ""
    echo -e "${BOLD}📦 INTERNAL MODULES (DO NOT RUN DIRECTLY):${NC}"
    echo "    scripts/test-utils.sh        → shared utilities, sourced by run-tests.sh"
    echo "    scripts/test-lint.sh         → lint phase definitions"
    echo "    scripts/test-unit.sh         → unit test phase definitions"
    echo "    scripts/test-integration.sh  → integration/E2E phase definitions"
    echo "    scripts/debug-logs.sh        → log analysis functions"
    echo "    setup-test-env.sh            → DB isolation setup (called automatically)"
    echo ""
    echo -e "${BOLD}⚠️ RULES FOR AI AGENTS:${NC}"
    echo "    1. ALWAYS use ./run-tests.sh — NEVER run scripts/ files directly"
    echo "    2. Use --dry-run to verify what will run before executing"
    echo "    3. Use --test-only + --test=NAME for fast iteration on a single test"
    echo "    4. Use --docker=dev exec SERVICE bash to run ad-hoc commands in containers"
    echo "    5. Test logs are stored in a volume. Use --logs or --tail-logs to read them"
    echo "    6. Cache persists across runs. Use --clean only when something is truly broken"
}

# --- Docker env command handler (must run before sourcing modules) ---
# Handles: ./run-tests.sh --docker=ENV [podman-compose subcommand...]
for _arg in "$@"; do
    if [[ "$_arg" == --docker=* ]]; then
        _env="${_arg#--docker=}"
        _PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
        case "$_env" in
            dev)
                _files="-p leeattend-dev -f $_PROJECT_ROOT/docker-compose.yml -f $_PROJECT_ROOT/.compose/docker-compose.dev.yml"
                ;;
            test)
                _files="-p leeattend-test -f $_PROJECT_ROOT/.compose/docker-compose.test.yml"
                ;;
            prod|base)
                _files="-p leeattend -f $_PROJECT_ROOT/docker-compose.yml"
                ;;
            *)
                echo -e "${RED}Error: Unknown environment '$_env'. Use: dev, test, prod${NC}"
                echo "Usage: ./run-tests.sh --docker=ENV [podman-compose subcommand...]"
                exit 1
                ;;
        esac
        # Collect remaining args (everything except --docker=ENV)
        _docker_args=()
        for _a in "$@"; do
            [[ "$_a" == --docker=* ]] && continue
            _docker_args+=("$_a")
        done
        echo "Running: podman-compose $_files ${_docker_args[*]}"
        podman-compose $_files "${_docker_args[@]}" </dev/null
        exit $?
    fi
done

# ── Skip PTY check for utility commands ──────────────────────────────
for _arg in "$@"; do
    case "$_arg" in --db-backup|--db-restore=*|--help) _PTY_CHECK_DONE=1; break;; esac
done

# ── PTY Safety Gate ──────────────────────────────────────────────────
# Run STATIC scan only before anything else. Runtime tests (which use
# the `script` command) are excluded here because `script` creates PTY
# child processes that themselves leak FDs in git hook context.
# For full runtime tests, run: ./scripts/test-pty-safety.sh
if [[ -z "${_PTY_CHECK_DONE:-}" && -f "$SCRIPT_DIR/scripts/test-pty-safety.sh" ]]; then
    echo "🔒 Running PTY safety check (static)..."
    if ! "$SCRIPT_DIR/scripts/test-pty-safety.sh" --static; then
        echo "❌ PTY safety check failed — fix issues above before running tests."
        exit 1
    fi
    echo ""
fi

parse_cli_args() {
    # --- Main Logic ---
    
    GLOBAL_TIMEOUT=${TIMEOUT_GLOBAL:-1800}
    
    if [[ $# -eq 0 ]]; then
        echo -e "${RED}Error: No arguments provided.${NC}" >&2
        echo ""
        show_help
        exit 1
    fi
    
    FILTER=""
    MODE=""
    PATTERN=""
    COMPONENT=""
    PHASE_LINT=false
    PHASE_UNIT=false
    PHASE_INTEGRATION=false
    COVERAGE=false
    TEST_ONLY=false
    LIST_TESTS=false
    DRY_RUN=false
    SMOKE=false
    WARM_CACHE=false
    CACHE_STATS=false
    NO_CACHE=false
    SHOW_LOGS=false
    TAIL_LOGS=false
    DEV_LOGS=""
    SHARD_N=""
    SHARD_M=""
    
    # Parse flags and find mode/pattern
    for arg in "$@"; do
        if [[ "$arg" == --timeout=* ]]; then
            GLOBAL_TIMEOUT="${arg#*=}"
            # Cap all phase timeouts by global timeout to ensure it's respected
            TIMEOUT_LINT=$GLOBAL_TIMEOUT
            TIMEOUT_UNIT=$GLOBAL_TIMEOUT
            TIMEOUT_INTEGRATION=$GLOBAL_TIMEOUT
            TIMEOUT_GLOBAL=$GLOBAL_TIMEOUT
        elif [[ "$arg" == --test=* ]]; then
            PATTERN="${arg#*=}"
        elif [[ "$arg" == --component=* ]]; then
            COMPONENT="${arg#*=}"
        elif [[ "$arg" == --shard=* ]]; then
            # FIX: Cannot use `local` outside a function in bash.
            # `local shard_str=...` here would cause: "bash: local: can only be used in a function"
            # With `set -e` enabled, this error silently terminates the entire script.
            # Using a prefixed temp variable (_shard_str) instead.
            _shard_str="${arg#*=}"
            SHARD_N="${_shard_str%%/*}"
            SHARD_M="${_shard_str##*/}"
        elif [[ "$arg" == "--no-color" ]]; then
            export NO_COLOR=1
            GREEN='' RED='' YELLOW='' BLUE='' BOLD='' NC=''
        elif [[ "$arg" == --* ]]; then
            # Handle modes and phases
            case "$arg" in
                --all) MODE="all"; FILTER="";;
                --api) MODE="api"; FILTER="apiserver|agent";;
                --web) MODE="web"; FILTER="webdashboard";;
                --integration) MODE="integration"; FILTER="apiserver|agent";;
                --lint) PHASE_LINT=true;;
                --unit) PHASE_UNIT=true;;
                --integration-only) PHASE_INTEGRATION=true;;
                --coverage) COVERAGE=true;;
                --test-only) TEST_ONLY=true;;
                --list) LIST_TESTS=true;;
                --dry-run) DRY_RUN=true;;
                --smoke) SMOKE=true;;
                --warm-cache) WARM_CACHE=true;;
                --cache-stats) CACHE_STATS=true;;
                --no-cache) NO_CACHE=true;;
                --logs) SHOW_LOGS=true;;
                --tail-logs) TAIL_LOGS=true;;
                --dev-logs) DEV_LOGS=dump;;
                --dev-logs=follow) DEV_LOGS=follow;;
                --db-backup) db_backup; exit 0;;
                --db-restore=*) db_restore "${arg#*=}"; exit 0;;
                --clean)
                    echo -e "${BLUE}${BOLD}🧹 Cleaning up test containers and volumes...${NC}"
                    dc down --volumes --remove-orphans
                    rm -rf "$PROJECT_ROOT/target" "$PROJECT_ROOT/.sccache"
                    echo -e "${GREEN}✅ Cleanup complete!${NC}"
                    exit 0
                    ;;
                --clean-cache)
                    echo -e "${BLUE}${BOLD}🧹 Cleaning cache only (keeping containers)...${NC}"
                    rm -rf "$PROJECT_ROOT/target" "$PROJECT_ROOT/.sccache"
                    if [[ "${USE_DOCKER:-}" == "true" ]]; then
                        dc exec -T apiserver_test bash -c "rm -rf /workspace/target /sccache 2>/dev/null || true"
                        dc exec -T agent_test bash -c "rm -rf /workspace/target /sccache 2>/dev/null || true"
                        dc exec -T web_test bash -c "rm -rf /workspace/.npm-cache /workspace/webdashboard/node_modules 2>/dev/null || true"
                    fi
                    echo -e "${GREEN}✅ Cache cleaned!${NC}"
                    exit 0
                    ;;
                --help) show_help; exit 0;;
            esac
        else
            # If it's not a flag and we haven't set a pattern via --test, 
            # then it's the legacy positional pattern
            if [[ -z "$PATTERN" ]]; then
                PATTERN="$arg"
            fi
        fi
    done
    
    # Handle log extraction
    if [[ "$SHOW_LOGS" == "true" || "$TAIL_LOGS" == "true" ]]; then
        echo -e "${BLUE}${BOLD}📜 Reading test logs from .test_logs/...${NC}"
        if [[ "$TAIL_LOGS" == "true" ]]; then
            tail -f "$PROJECT_ROOT/.test_logs"/*.log </dev/null 2>/dev/null || echo "No logs found."
        else
            cat "$PROJECT_ROOT/.test_logs"/*.log </dev/null 2>/dev/null || echo "No logs found."
        fi
        exit 0
    fi
    
    # Handle dev service logs
    if [[ -n "$DEV_LOGS" ]]; then
        _log_dir="$PROJECT_ROOT/logs"
        if [[ ! -d "$_log_dir" ]] || ! ls "$_log_dir"/*.log &>/dev/null; then
            echo -e "${YELLOW}No service logs found in ./logs/ — start the dev env first:${NC}"
            echo "  ./run-tests.sh --docker=dev up -d"
            exit 1
        fi
        if [[ "$DEV_LOGS" == "follow" ]]; then
            echo -e "${BLUE}${BOLD}📜 Live-tailing service logs from ./logs/...${NC}"
            tail -f "$_log_dir"/*.log </dev/null
        else
            echo -e "${BLUE}${BOLD}📜 Service logs from ./logs/:${NC}"
            for f in "$_log_dir"/*.log; do
                echo -e "\n${YELLOW}═══ $(basename "$f") ═══${NC}"
                cat "$f"
            done
        fi
        exit 0
    fi
    
    if [[ -z "$MODE" ]]; then
        echo -e "${RED}Error: No mode specified (e.g., --all, --api, --web)${NC}"
        show_help
        exit 1
    fi
    
    # Validate shard parameters
    if [[ -n "$SHARD_N" || -n "$SHARD_M" ]]; then
        if [[ -z "$SHARD_N" || -z "$SHARD_M" || "$SHARD_N" -lt 1 || "$SHARD_M" -lt 1 || "$SHARD_N" -gt "$SHARD_M" ]]; then
            echo -e "${RED}Error: Invalid shard format. Use --shard=N/M (e.g., 1/3)${NC}"
            exit 1
        fi
        echo -e "${BLUE}${BOLD}🧩 Running shard $SHARD_N of $SHARD_M${NC}"
    fi
    
    # Apply component filter
    if [[ -n "$COMPONENT" ]]; then
        case "$COMPONENT" in
            apiserver)
                FILTER="apiserver"
                echo -e "${BLUE}🎯 Component filter: ${YELLOW}apiserver only${NC}"
                ;;
            agent|agent-main)
                FILTER="agent"
                echo -e "${BLUE}🎯 Component filter: ${YELLOW}agent-main only${NC}"
                ;;
            web|webdashboard)
                FILTER="webdashboard"
                echo -e "${BLUE}🎯 Component filter: ${YELLOW}webdashboard only${NC}"
                ;;
            *)
                echo -e "${RED}Error: Invalid component. Use: apiserver, agent-main, webdashboard${NC}"
                exit 1
                ;;
        esac
    fi
    
    # Handle --cache-stats
    if [[ "$CACHE_STATS" == "true" ]]; then
        echo -e "${BLUE}${BOLD}📊 Cache Statistics${NC}"
        echo ""
        
        if [[ "${USE_DOCKER:-}" == "true" ]]; then
            echo -e "${YELLOW}=== Cargo (apiserver) ===${NC}"
            dc exec -T apiserver_test bash -c "du -sh /workspace/target 2>/dev/null || echo 'No cache'" || true
            dc exec -T apiserver_test bash -c "sccache --show-stats 2>/dev/null | head -n 10 || echo 'sccache not available'" </dev/null || true
            
            echo ""
            echo -e "${YELLOW}=== Cargo (agent-main) ===${NC}"
            dc exec -T agent_test bash -c "du -sh /workspace/target 2>/dev/null || echo 'No cache'" || true
            dc exec -T agent_test bash -c "sccache --show-stats 2>/dev/null | head -n 10 || echo 'sccache not available'" </dev/null || true
            
            echo ""
            echo -e "${YELLOW}=== NPM ===${NC}"
            dc exec -T web_test bash -c "du -sh /workspace/.npm-cache 2>/dev/null || echo 'No cache'" || true
            dc exec -T web_test bash -c "du -sh /workspace/webdashboard/node_modules 2>/dev/null || echo 'No node_modules'" || true
        else
            echo -e "${YELLOW}=== Local Cargo ===${NC}"
            du -sh "$PROJECT_ROOT/target" 2>/dev/null || echo "No cache"
            
            echo ""
            echo -e "${YELLOW}=== Local NPM ===${NC}"
            du -sh "$HOME/.npm" 2>/dev/null || echo "No npm cache"
        fi
        
        exit 0
    fi
    
    # Apply test-only mode (skip lint/unit, run only pattern)
    if [[ "$TEST_ONLY" == "true" ]]; then
        PHASE_LINT=false
        PHASE_UNIT=false
        PHASE_INTEGRATION=true
        echo -e "${BLUE}⚡ Test-only mode: skipping lint and unit phases${NC}"
    fi
    
    # Apply smoke mode (lint + unit + fast smoke-tagged integration)
    if [[ "$SMOKE" == "true" ]]; then
        PHASE_LINT=true
        PHASE_UNIT=true
        PHASE_INTEGRATION=true
        # Set the integration test filter to "smoke" — nextest uses substring
        # matching on test names, and Playwright uses --grep to filter titles.
        if [[ -z "$PATTERN" ]]; then
            PATTERN="smoke"
        fi
        echo -e "${BLUE}🔥 Smoke mode: running lint + unit + smoke-tagged integration${NC}"
    fi
    
    # Set default phases if not specified
    if [[ "$PHASE_LINT" == "false" && "$PHASE_UNIT" == "false" && "$PHASE_INTEGRATION" == "false" ]]; then
        if [[ "$MODE" != "integration" ]]; then
            PHASE_LINT=true
            PHASE_UNIT=true
        fi
        PHASE_INTEGRATION=true
    fi
    
    # Handle --dry-run (show what would execute)
    if [[ "$DRY_RUN" == "true" ]]; then
        echo -e "${BLUE}${BOLD}🔍 Dry-run: showing what would execute...${NC}"
        echo -e "${BLUE}Mode: $MODE${NC}"
        echo -e "${BLUE}Filter: $FILTER${NC}"
        echo -e "${BLUE}Pattern: ${PATTERN:-none}${NC}"
        echo -e "${BLUE}Component: ${COMPONENT:-all}${NC}"
        echo -e "${BLUE}Phases:${NC}"
        [[ "$PHASE_LINT" == "true" ]] && echo -e "  ✓ Lint"
        [[ "$PHASE_UNIT" == "true" ]] && echo -e "  ✓ Unit Tests"
        [[ "$PHASE_INTEGRATION" == "true" ]] && echo -e "  ✓ Integration Tests"
        [[ "$COVERAGE" == "true" ]] && echo -e "  ✓ Coverage (cargo llvm-cov / vitest)"
        echo -e "${GREEN}💡 This is a dry-run. No tests were executed.${NC}"
        exit 0
    fi
    
    # Handle --list (show available tests)
    if [[ "$LIST_TESTS" == "true" ]]; then
        echo -e "${BLUE}${BOLD}📋 Listing available tests...${NC}"
        if [[ "$MODE" == "api" || "$MODE" == "all" ]]; then
            echo -e "\n${YELLOW}=== API Tests (apiserver) ===${NC}"
            echo "  cargo nextest list --message-format=plain 2>/dev/null || echo 'Run in container for full list'"
        fi
        if [[ "$MODE" == "web" || "$MODE" == "all" ]]; then
            echo -e "\n${YELLOW}=== Web Tests (webdashboard) ===${NC}"
            echo "  npm test -- --list 2>/dev/null || echo 'Run in container for full list'"
        fi
        echo -e "${GREEN}💡 Tip: Run with USE_DOCKER=true to get actual list${NC}"
        exit 0
    fi
    
    echo -e "${BLUE}${BOLD}🚀 Starting LeeAttend Test Suite in [$MODE] mode${NC}"
    echo -e "${BLUE}⏱️ Global timeout: ${GLOBAL_TIMEOUT}s ($(( GLOBAL_TIMEOUT / 60 )) min)${NC}"
    if [[ -n "$PATTERN" ]]; then
        echo -e "${BLUE}🎯 Specific test pattern: ${YELLOW}$PATTERN${NC}"
    fi
    echo -e "${BLUE}================================================================${NC}"
    
    # Export timeout vars for sub-scripts
    export TIMEOUT_GLOBAL="${TIMEOUT_GLOBAL:-1800}"
    export TIMEOUT_LINT="${TIMEOUT_LINT:-900}"
    export TIMEOUT_UNIT="${TIMEOUT_UNIT:-900}"
    export TIMEOUT_INTEGRATION="${TIMEOUT_INTEGRATION:-1200}"
    export TIMEOUT_BUILD="${TIMEOUT_BUILD:-600}"
    export TIMEOUT_DB_WAIT="${TIMEOUT_DB_WAIT:-120}"
    export TIMEOUT_WEB_READY="${TIMEOUT_WEB_READY:-300}"
    export SHARD_N="$SHARD_N"
    export SHARD_M="$SHARD_M"
    
    # Handle --no-cache
    if [[ "$NO_CACHE" == "true" ]]; then
        echo -e "${YELLOW}⚠️ Cache disabled - forcing fresh build${NC}"
        unset RUSTC_WRAPPER
        export CARGO_NET_OFFLINE=true
        export NPM_CONFIG_CACHE=/dev/null
    fi
    
    # Handle --warm-cache (pre-compile dependencies)
    if [[ "$WARM_CACHE" == "true" ]]; then
        echo -e "${BLUE}${BOLD}🔥 Warming up cache - pre-compiling dependencies...${NC}"
        
        if [[ "${USE_DOCKER:-}" == "true" ]]; then
            # Start containers
            dc build apiserver_test agent_test web_test
            dc up -d apiserver_test agent_test web_test
            
            echo -e "${BLUE}📦 Pre-compiling apiserver...${NC}"
            dc exec -T apiserver_test bash -c "cargo check --lib 2>/dev/null || true" </dev/null &
            pid_api=$!
            
            echo -e "${BLUE}📦 Pre-compiling agent-main...${NC}"
            dc exec -T agent_test bash -c "cargo check --lib 2>/dev/null || true" </dev/null &
            pid_agent=$!
            
            echo -e "${BLUE}📦 Installing web dependencies...${NC}"
            dc exec -T web_test npm install --no-audit --no-fund </dev/null &
            pid_web=$!
            
            # Wait for explicit PIDs to complete
            wait $pid_api $pid_agent $pid_web
            
            echo -e "${GREEN}✅ Cache warmed up!${NC}"
            echo -e "${BLUE}💡 Run tests now: ./run-tests.sh --api${NC}"
        else
            echo -e "${YELLOW}⚠️ Warm-cache only works with containers. Use --help for options.${NC}"
        fi
        exit 0
    fi
    
    # Clear previous results
    if [[ -L "$PROJECT_ROOT/.test_logs" ]]; then
        mkdir -p "$(readlink "$PROJECT_ROOT/.test_logs")"
    else
        mkdir -p "$PROJECT_ROOT/.test_logs"
    fi
    rm -f "$PROJECT_ROOT/.test_logs/results.txt"
    
    # NOTE: Global watchdog removed. Per-phase timeouts (TIMEOUT_LINT,
    # TIMEOUT_UNIT, TIMEOUT_INTEGRATION, TIMEOUT_BUILD) already protect
    # each stage individually. The background watchdog process caused PTY
    # file descriptor inheritance issues when run-tests.sh was invoked from
    # git hooks or AI agent terminals, making commands appear to hang
    # indefinitely even after successful completion.
}
