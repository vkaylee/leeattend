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
"$SCRIPT_DIR/_cargo.sh" run --bin apiserver --quiet -- gen-openapi > "$PROJECT_ROOT/webdashboard/openapi.raw.json"
# Strip container logs (everything before the first '{')
sed -n '/^{/,$p' "$PROJECT_ROOT/webdashboard/openapi.raw.json" > "$PROJECT_ROOT/webdashboard/openapi.json"
rm -f "$PROJECT_ROOT/webdashboard/openapi.raw.json"

# 2. Generate TypeScript types for Frontend using openapi-typescript
echo "💅 Generating API SDK for Frontend using openapi-typescript..."
cd "$PROJECT_ROOT/webdashboard" && "../scripts/_npm.sh" run gen-api && rm -f openapi.json

echo "✅ API Synchronization complete!"

