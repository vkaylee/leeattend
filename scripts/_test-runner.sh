#!/bin/bash

# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e "\033[0;31m❌ SECURITY VIOLATION: Direct execution of this module is forbidden.\033[0m"
    echo -e "👉 This script is an internal module. Use: \033[1m./test.sh\033[0m"
    exit 1
fi


# Guard: Prevent direct execution. Must be sourced or run via official entry point.
if [[ "" == "/bin/bash" ]]; then
    echo -e "[0;31m❌ SECURITY VIOLATION: Direct execution of this module is forbidden.[0m"
    echo -e "👉 This script is an internal module. Use: [1m./test.sh[0m"
    exit 1
fi


# ── Profile helpers ─────────────────────────────────────────────────
# Map services → docker-compose profiles, so `dc up` only starts what
# the current phase actually needs instead of launching the full stack.
#
# Service ➜ profile
#   db, redis, db_init, apiserver, agent-main  →  backend
#   bun_init, webdashboard                      →  frontend

_service_to_profile() {
    case "$1" in
        db|redis|db_init|apiserver|agent-main) echo "backend" ;;
        bun_init|webdashboard)                 echo "frontend" ;;
    esac
}

# services_to_profile_flags "webdashboard bun_init"
#   → "--profile frontend"
services_to_profile_flags() {
    local flags=""
    local have_backend=false have_frontend=false
    for svc in $1; do
        case "$(_service_to_profile "$svc")" in
            backend)  have_backend=true  ;;
            frontend) have_frontend=true ;;
        esac
    done
    $have_backend  && flags="$flags --profile backend"
    $have_frontend && flags="$flags --profile frontend"
    echo "$flags"
}

run_lint() {
    local filter=$1
    echo -e "\n${BLUE}🔹 Running Linting [$filter]...${NC}"

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        echo -e "${BLUE}🏗️ Ensuring containers are up for Linting...${NC}"
        local services=""
        local need_web=false
        
        if [[ "$filter" == *"web"* ]]; then
            # We need apiserver for api-sync even if testing web
            services="webdashboard apiserver bun_init db_init db redis"
            need_web=true
        elif [[ "$filter" == *"apiserver"* || "$filter" == *"agent"* || "$filter" == *"api"* ]]; then
            services="apiserver agent-main db_init db redis"
        else
            services="apiserver agent-main webdashboard db_init bun_init db redis"
            need_web=true
        fi
        
        local profile_flags=$(services_to_profile_flags "$services")
        dc $profile_flags build $services
        dc $profile_flags up -d
        # Migrations must run before any Rust compilation (sqlx verifies queries at compile time).
        # Even --web mode needs this because api-sync runs `cargo run -- gen-openapi`.
        # Compose handles this automatically via the db_init and bun_init services.
    fi

    leeattend_run_lint "$filter"
}

run_unit() {
    local filter=$1
    echo -e "\n${BLUE}🔹 Running Unit Tests [$filter]...${NC}"
    
    if [[ "$USE_DOCKER" == "true" ]]; then
        echo -e "${BLUE}🏗️ Ensuring containers are up...${NC}"
        local services=""
        local need_web=false
        
        if [[ "$filter" == *"web"* ]]; then
            services="webdashboard bun_init db redis"
            need_web=true
        elif [[ "$filter" == *"apiserver"* || "$filter" == *"agent"* || "$filter" == *"api"* ]]; then
            services="apiserver agent-main db_init db redis"
        else
            services="apiserver agent-main webdashboard db_init bun_init db redis"
            need_web=true
        fi
        
        local profile_flags=$(services_to_profile_flags "$services")
        dc $profile_flags build $services
        dc $profile_flags up -d
        # Migrations must run before cargo compile (sqlx verifies queries at compile time)
        # Compose handles this automatically.
    fi
    
    leeattend_run_unit "$filter"
}

run_integration() {
    local filter=$1

    # Integration tests use isolated DBs — parallel is safe, keep auto-detected NUM_WORKERS
    echo -e "\n${BLUE}🔹 Running Integration & E2E Tests [$filter]...${NC}"
    
    if [[ "$USE_DOCKER" == "true" ]]; then
        echo -e "${BLUE}🏗️ Ensuring environment for Integration tests...${NC}"
        local services=""
        local need_web=false
        
        if [[ "$filter" == *"web"* ]]; then
            # Web E2E dùng Mock API, KHÔNG cần DB và Backend thật
            services="webdashboard bun_init"
            need_web=true
        else
            # Integration tests cho API bắt buộc cần DB thật
            services="db redis apiserver agent-main db_init"
            if [[ "$filter" == "" || "$filter" == "all" || "$filter" == "api" ]]; then 
                services="$services webdashboard bun_init"
                need_web=true
            fi
        fi

        local profile_flags=$(services_to_profile_flags "$services")
        dc $profile_flags build $services
        dc $profile_flags up -d
        # Compose will automatically wait for bun_init
    fi

    if [[ "$filter" == *"web"* ]]; then
        # Chạy thẳng E2E không cần setup-test-env (vì không cần DB isolated cho từng worker)
        leeattend_run_integration "$filter"
    else
        # Integration tests cho API vẫn cần môi trường thực
        ./setup-test-env.sh leeattend_run_integration "$filter"
    fi
}
