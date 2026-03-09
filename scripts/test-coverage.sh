#!/bin/bash

# ==============================================================================
# ⚠️ INTERNAL MODULE — DO NOT RUN DIRECTLY
# Sourced by: ./run-tests.sh
# ==============================================================================
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "❌ This script is an internal module. Do not run directly."
    echo "👉 Use: ./run-tests.sh --api --coverage  (or --all, --web)"
    exit 1
fi
# ==============================================================================

# LeeAttend Coverage Methods
# ==============================================================================
# Uses:
#   Rust  → cargo llvm-cov nextest  (requires llvm-tools-preview + cargo-llvm-cov)
#   Web   → npm run test:coverage   (vitest --coverage, already wired in package.json)
#
# Outputs (inside container, bind-mounted to host via .:/workspace):
#   .test_logs/coverage-apiserver.lcov
#   .test_logs/coverage-apiserver-html/
#   .test_logs/coverage-agent.lcov
#   .test_logs/coverage-agent-html/
#   webdashboard/coverage/   (vitest default output dir)
# ==============================================================================

leeattend_run_coverage() {
    local filter_input="${1:-}"

    # Extract component filter (ignore test pattern for coverage — always run all)
    local COMPONENT_FILTER
    COMPONENT_FILTER=$(echo "$filter_input" | cut -d':' -f1)

    echo -e "\n${BLUE}${BOLD}📊 Running Test Coverage [$COMPONENT_FILTER]...${NC}"

    # --- Rust coverage commands ---
    # Disable sccache wrapper: llvm-cov instruments at compile time and is incompatible
    # with sccache when generating coverage data.
    local COVERAGE_ENV="RUSTC_WRAPPER=''"

    local APISERVER_COV_CMD
    local AGENT_COV_CMD
    local WEB_COV_CMD

    if [[ "${USE_DOCKER:-}" == "true" ]]; then
        # apiserver — lcov + html report
        APISERVER_COV_CMD="$DOCKER_COMPOSE_CMD exec -T apiserver_test bash -c \
\"mkdir -p /workspace/.test_logs && \
$COVERAGE_ENV cargo llvm-cov nextest --lib --test integration \
  --lcov --output-path /workspace/.test_logs/coverage-apiserver.lcov \
  --no-tests=pass 2>&1 && \
$COVERAGE_ENV cargo llvm-cov report --html \
  --output-dir /workspace/.test_logs/coverage-apiserver-html \
2>&1 && \
$COVERAGE_ENV cargo llvm-cov report \
  --summary-only 2>&1\""

        # agent-main — lcov + html report
        AGENT_COV_CMD="$DOCKER_COMPOSE_CMD exec -T \
-e CENTRAL_SERVER_URL=ws://unit-test-invalid:3000 \
agent_test bash -c \
\"mkdir -p /workspace/.test_logs && \
$COVERAGE_ENV cargo llvm-cov nextest --lib \
  --lcov --output-path /workspace/.test_logs/coverage-agent.lcov \
  --no-tests=pass 2>&1 && \
$COVERAGE_ENV cargo llvm-cov report --html \
  --output-dir /workspace/.test_logs/coverage-agent-html \
2>&1 && \
$COVERAGE_ENV cargo llvm-cov report \
  --summary-only 2>&1\""

        # web — vitest coverage (outputs to webdashboard/coverage/)
        WEB_COV_CMD="$DOCKER_COMPOSE_CMD exec -T web_test npm run test:coverage"
    else
        # Local (non-Docker) fallback
        APISERVER_COV_CMD="cd apiserver && $COVERAGE_ENV cargo llvm-cov nextest --lib --test integration \
  --lcov --output-path ../.test_logs/coverage-apiserver.lcov --no-tests=pass && \
$COVERAGE_ENV cargo llvm-cov report --html \
  --output-dir ../.test_logs/coverage-apiserver-html && \
$COVERAGE_ENV cargo llvm-cov report --summary-only"

        AGENT_COV_CMD="cd agent-main && CENTRAL_SERVER_URL=ws://unit-test-invalid:3000 \
$COVERAGE_ENV cargo llvm-cov nextest --lib \
  --lcov --output-path ../.test_logs/coverage-agent.lcov --no-tests=pass && \
$COVERAGE_ENV cargo llvm-cov report --html \
  --output-dir ../.test_logs/coverage-agent-html && \
$COVERAGE_ENV cargo llvm-cov report --summary-only"

        WEB_COV_CMD="cd webdashboard && npm run test:coverage"
    fi

    run_parallel_ordered "Coverage" "$COMPONENT_FILTER" \
        "1|apiserver|apiserver|$APISERVER_COV_CMD" \
        "2|agent-main|agent-main|$AGENT_COV_CMD" \
        "4|webdashboard|webdashboard|$WEB_COV_CMD"

    echo -e "\n${GREEN}${BOLD}📁 Coverage reports saved to:${NC}"
    if [[ "$COMPONENT_FILTER" != *"web"* ]]; then
        echo -e "   ${BLUE}Rust LCOV  :${NC} .test_logs/coverage-apiserver.lcov"
        echo -e "   ${BLUE}Rust HTML  :${NC} .test_logs/coverage-apiserver-html/index.html"
        echo -e "   ${BLUE}Agent LCOV :${NC} .test_logs/coverage-agent.lcov"
        echo -e "   ${BLUE}Agent HTML :${NC} .test_logs/coverage-agent-html/index.html"
    fi
    if [[ "$COMPONENT_FILTER" == *"web"* || "$COMPONENT_FILTER" == "" || "$COMPONENT_FILTER" == "all" ]]; then
        echo -e "   ${BLUE}Web HTML   :${NC} webdashboard/coverage/index.html"
    fi
}
