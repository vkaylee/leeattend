#!/bin/bash
set -e

# Load project environment
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC2034
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# shellcheck source=/dev/null
source "$SCRIPT_DIR/_compose-env.sh"
# shellcheck source=/dev/null
source "$SCRIPT_DIR/_docker-ops.sh"

echo "🔄 Synchronizing API types..."

# 1. Generate OpenAPI spec from Backend
echo "📡 Generating OpenAPI spec from Backend..."
dc exec -T apiserver bash -c "cargo run --quiet -- gen-openapi > /workspace/shared/openapi.json" </dev/null

# 2. Generate TypeScript types for Frontend
echo "💅 Generating TypeScript types for Frontend..."
dc exec -T webdashboard bash -c "npx openapi-typescript /workspace/shared/openapi.json -o ./src/types/api-generated.ts" </dev/null

echo "✅ API Synchronization complete!"
