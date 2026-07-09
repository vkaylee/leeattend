#!/bin/bash

# ==============================================================================
# Cloudflare Pages Deployment Script (Hermetic)
# Usage: ./scripts/deploy-dashboard.sh [--env <path-to-env-file>]
# ==============================================================================

set -e

# Path to this script
SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
PROJECT_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
export PROJECT_ROOT

# Default values
ENV_FILE=""

show_usage() {
    echo "Usage: $0 [--env <path-to-env-file>]"
    echo ""
    echo "Options:"
    echo "  --env        Optional path to the environment variables file for deployment"
    echo "  -h, --help   Show this help message"
    echo ""
    echo "Example: $0 --env webdashboard/.env.production"
}

# Parse arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --env) ENV_FILE="$2"; shift ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *) 
            echo "❌ Error: Unknown parameter passed: $1"
            echo ""
            show_usage
            exit 1 
            ;;
    esac
    shift
done

# Source env file if provided
ENV_FILE_ABS=""
if [ -n "$ENV_FILE" ]; then
    if [ ! -f "$ENV_FILE" ]; then
        echo "❌ Error: Environment file '$ENV_FILE' not found."
        exit 1
    fi
    echo "📍 Sourcing env file: $ENV_FILE"
    ENV_FILE_ABS=$(readlink -f "$ENV_FILE")
    set -a
    # shellcheck source=/dev/null
    source "$ENV_FILE_ABS"
    set +a
fi

# Validate required variables (either from environment or from sourced env file)
check_required_vars() {
    local missing=()
    [ -z "$CLOUDFLARE_API_TOKEN" ] && missing+=("CLOUDFLARE_API_TOKEN")
    [ -z "$CLOUDFLARE_ACCOUNT_ID" ] && missing+=("CLOUDFLARE_ACCOUNT_ID")
    [ -z "$CLOUDFLARE_PROJECT_NAME" ] && missing+=("CLOUDFLARE_PROJECT_NAME")
    
    if [ ${#missing[@]} -ne 0 ]; then
        echo "❌ Error: Missing required Cloudflare variables: ${missing[*]}"
        echo "Either provide --env <file> or set them in the environment."
        exit 1
    fi
}

check_required_vars

echo "🚀 Starting Cloudflare Pages deployment for webdashboard..."

cd "$PROJECT_ROOT"

# Common podman-compose command
COMPOSE_CMD=(
    podman-compose
    -p leeattend-test
)

# Only add --env-file if it exists
if [ -n "$ENV_FILE_ABS" ]; then
    COMPOSE_CMD+=(--env-file "$ENV_FILE_ABS")
fi

COMPOSE_CMD+=(
    -f .compose/docker-compose.test.yml
    --profile frontend
    run -T --rm --no-deps
    -e "CLOUDFLARE_API_TOKEN=$CLOUDFLARE_API_TOKEN"
    -e "CLOUDFLARE_ACCOUNT_ID=$CLOUDFLARE_ACCOUNT_ID"
    -e "CLOUDFLARE_PROJECT_NAME=$CLOUDFLARE_PROJECT_NAME"
    -e "CLOUDFLARE_BRANCH=${CLOUDFLARE_BRANCH:-main}"
    -e "CI=true"
    -e "WRANGLER_SEND_METRICS=false"
    -e "NODE_OPTIONS=--max_old_space_size=4096"
)

# Dynamically add all VITE_* environment variables so Vite can bake them into the build
# compgen might not be available in all shells, but bash usually has it.
for var in $(compgen -v | grep '^VITE_'); do
    COMPOSE_CMD+=("-e" "$var=${!var}")
done

COMPOSE_CMD+=("--entrypoint")

# Step 1: Install dependencies
echo "📦 [1/3] Installing dependencies..."
"${COMPOSE_CMD[@]}" bash webdashboard -c "rm -rf node_modules/.[!.]* node_modules/* && bun install --frozen-lockfile" </dev/null


# Step 2: Build the project
echo "🏗️ [2/3] Building webdashboard..."
"${COMPOSE_CMD[@]}" bash webdashboard -c "NODE_ENV=production bun run build" </dev/null

# Step 3: Deploy to Cloudflare Pages
echo "☁️ [3/3] Deploying to Cloudflare Pages..."
# Ensure the project exists before deploying to prevent interactive prompts
"${COMPOSE_CMD[@]}" bash webdashboard -c "bunx wrangler pages project create \"\$CLOUDFLARE_PROJECT_NAME\" --production-branch \"\${CLOUDFLARE_BRANCH:-main}\" || true" </dev/null
# We use a bash wrapper inside the container to ensure environment variables are expanded correctly
"${COMPOSE_CMD[@]}" bash webdashboard -c "bunx wrangler pages deploy dist --project-name \"\$CLOUDFLARE_PROJECT_NAME\" --branch \"\${CLOUDFLARE_BRANCH:-main}\"" </dev/null

# Step 4: Purge Cloudflare edge cache so the new deployment is served
# immediately. Without this, edge nodes may keep serving the stale index.html
# (referencing old asset hashes) until the cache expires, causing white-screen
# errors on custom domains. Requires CLOUDFLARE_ZONE_ID + a token with the
# Cache Purge permission. Skipped gracefully if the zone id is unset.
echo ""
if [ -n "${CLOUDFLARE_ZONE_ID:-}" ]; then
  echo "🧹 [4/4] Purging Cloudflare cache for zone ${CLOUDFLARE_ZONE_ID}..."
  PURGE_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
    "https://api.cloudflare.com/client/v4/zones/${CLOUDFLARE_ZONE_ID}/purge_cache" \
    -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"purge_everything": true}')
  PURGE_STATUS=$(echo "$PURGE_RESPONSE" | tail -1)
  PURGE_BODY=$(echo "$PURGE_RESPONSE" | sed '$d')
  if [ "$PURGE_STATUS" = "200" ]; then
    echo "  ✅ Cache purged successfully."
  else
    echo "  ⚠️  Cache purge returned HTTP ${PURGE_STATUS}: ${PURGE_BODY}"
    echo "     The deploy itself succeeded — cache will expire naturally."
  fi
else
  echo "ℹ️  [4/4] Skipping cache purge (set CLOUDFLARE_ZONE_ID to enable)."
  echo "   Note: without purge, custom domains may show a stale page until the"
  echo "   edge cache expires. Purge manually in the Cloudflare dashboard."
fi

echo ""
echo "✅ Deployment process completed successfully!"
