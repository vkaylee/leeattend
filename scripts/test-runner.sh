#!/bin/bash

run_lint() {
    local filter=$1
    echo -e "\n${BLUE}🔹 Running Linting [$filter]...${NC}"

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        echo -e "${BLUE}🏗️ Ensuring containers are up for Linting...${NC}"
        local services=""
        local need_web=false
        
        if [[ "$filter" == *"web"* ]]; then
            # We need apiserver_test for api-sync even if testing web
            services="web_test apiserver_test npm_init db_init test_db test_redis"
            need_web=true
        elif [[ "$filter" == *"apiserver"* || "$filter" == *"agent"* || "$filter" == *"api"* ]]; then
            services="apiserver_test agent_test db_init test_db test_redis"
        else
            services="apiserver_test agent_test web_test db_init npm_init test_db test_redis"
            need_web=true
        fi
        
        dc build $services
        dc up -d
        # Migrations must run before any Rust compilation (sqlx verifies queries at compile time).
        # Even --web mode needs this because api-sync runs `cargo run -- gen-openapi`.
        # Compose handles this automatically via the db_init and npm_init services.
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
            services="web_test npm_init test_db test_redis"
            need_web=true
        elif [[ "$filter" == *"apiserver"* || "$filter" == *"agent"* || "$filter" == *"api"* ]]; then
            # Unit tests cho API/Agent thường không cần DB thật (dùng memory), 
            # nhưng ta bật apiserver_test/agent_test container để chạy code trong đó
            services="apiserver_test agent_test db_init test_db test_redis"
        else
            services="apiserver_test agent_test web_test db_init npm_init test_db test_redis"
            need_web=true
        fi
        
        dc build $services
        dc up -d
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
            services="web_test npm_init"
            need_web=true
        else
            # Integration tests cho API bắt buộc cần DB thật
            services="test_db test_redis apiserver_test agent_test db_init"
            if [[ "$filter" == "" || "$filter" == "all" || "$filter" == "api" ]]; then 
                services="$services web_test npm_init"
                need_web=true
            fi
        fi

        dc build $services
        dc up -d
        # Compose will automatically wait for npm_init
    fi

    if [[ "$filter" == *"web"* ]]; then
        # Chạy thẳng E2E không cần setup-test-env (vì không cần DB isolated cho từng worker)
        leeattend_run_integration "$filter"
    else
        # Integration tests cho API vẫn cần môi trường thực
        ./setup-test-env.sh leeattend_run_integration "$filter"
    fi
}
