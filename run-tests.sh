#!/bin/bash

# ==============================================================================
# Unified Test Orchestrator for LeeAttend
# Delegates to specific test scripts.
# ==============================================================================

set -euo pipefail

# ANSI Color Codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}${BOLD}🚀 LeeAttend Unified Test Runner${NC}"

echo -e "\n${BLUE}🔹 Running Linting...${NC}"
./scripts/test-lint.sh

echo -e "\n${BLUE}🔹 Running Unit Tests...${NC}"
./scripts/test-unit.sh

echo -e "\n${BLUE}🔹 Running Integration Tests (with DB setup)...${NC}"
./setup-test-env.sh ./scripts/test-integration.sh

echo -e "\n${GREEN}${BOLD}🎉 All tests passed successfully!${NC}"