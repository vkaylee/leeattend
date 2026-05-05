#!/bin/bash

# ==============================================================================
# Cloudflare Pages Deployment Script (Hermetic)
# Usage: ./scripts/deploy-dashboard.sh --env <path-to-env-file>
# ==============================================================================

set -e

# Path to this script
SCRIPT_DIR=$(dirname "$(readlink -f "$0")")
PROJECT_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
export PROJECT_ROOT

# Default values
ENV_FILE=""

show_usage() {
    echo "Usage: $0 --env <path-to-env-file>"
    echo ""
    echo "Options:"
    echo "  --env        Path to the environment variables file for deployment"
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

if [ -z "$ENV_FILE" ]; then
    echo "❌ Error: --env <path-to-env-file> is required."
    echo ""
    show_usage
    exit 1
fi

if [ ! -f "$ENV_FILE" ]; then
    echo "❌ Error: Environment file '$ENV_FILE' not found."
    exit 1
fi

# Convert to absolute path for podman-compose
ENV_FILE_ABS=$(readlink -f "$ENV_FILE")

# Source the env file locally to validate and export required variables
# This will inject them into the script's environment so they can be passed to the container
set -a
source "$ENV_FILE_ABS"
set +a

MISSING_VARS=()
[ -z "$CLOUDFLARE_API_TOKEN" ] && MISSING_VARS+=("CLOUDFLARE_API_TOKEN")
[ -z "$CLOUDFLARE_ACCOUNT_ID" ] && MISSING_VARS+=("CLOUDFLARE_ACCOUNT_ID")
[ -z "$CLOUDFLARE_PROJECT_NAME" ] && MISSING_VARS+=("CLOUDFLARE_PROJECT_NAME")

if [ ${#MISSING_VARS[@]} -ne 0 ]; then
    echo "❌ Error: Missing required variables in $ENV_FILE:"
    for var in "${MISSING_VARS[@]}"; do
        echo "   - $var"
    done
    exit 1
fi

echo "🚀 Starting Cloudflare Pages deployment for webdashboard..."
echo "📍 Env file: $ENV_FILE"

cd "$PROJECT_ROOT"

# Common podman-compose command. Explicitly pass Cloudflare variables to the container.
COMPOSE_CMD=(
    podman-compose # </dev/null
    -p leeattend-test
    --env-file "$ENV_FILE_ABS"
    -f docker-compose.yml
    -f .compose/docker-compose.test.yml
    run -T --rm
    -e "CLOUDFLARE_API_TOKEN=$CLOUDFLARE_API_TOKEN"
    -e "CLOUDFLARE_ACCOUNT_ID=$CLOUDFLARE_ACCOUNT_ID"
    -e "CLOUDFLARE_PROJECT_NAME=$CLOUDFLARE_PROJECT_NAME"
    -e "CLOUDFLARE_BRANCH=$CLOUDFLARE_BRANCH"
    -e "CI=true"
    -e "WRANGLER_SEND_METRICS=false"
    -e "NODE_OPTIONS=--max_old_space_size=4096"
)

# Dynamically add all VITE_* environment variables so Vite can bake them into the build
for var in $(compgen -v | grep '^VITE_'); do
    COMPOSE_CMD+=("-e" "$var=${!var}")
done

COMPOSE_CMD+=("--entrypoint")

# Step 1: Install dependencies
echo "📦 [1/3] Installing dependencies..."
"${COMPOSE_CMD[@]}" bash webdashboard -c "npm install -g bun && bun install --frozen-lockfile" </dev/null

# Step 2: Build the project
echo "🏗️ [2/3] Building webdashboard..."
# Vite will automatically pick up VITE_* variables from the environment injected by --env-file
"${COMPOSE_CMD[@]}" bash webdashboard -c "npm install -g bun && bun run build" </dev/null

# Step 3: Deploy to Cloudflare Pages
echo "☁️ [3/3] Deploying to Cloudflare Pages..."
# Ensure the project exists before deploying to prevent interactive prompts
"${COMPOSE_CMD[@]}" bash webdashboard -c "npx wrangler pages project create \"\$CLOUDFLARE_PROJECT_NAME\" --production-branch \"\${CLOUDFLARE_BRANCH:-main}\" || true" </dev/null
# We use a bash wrapper inside the container to ensure environment variables are expanded correctly
"${COMPOSE_CMD[@]}" bash webdashboard -c "npx wrangler pages deploy dist --project-name \"\$CLOUDFLARE_PROJECT_NAME\" --branch \"\${CLOUDFLARE_BRANCH:-main}\"" </dev/null

echo ""
echo "✅ Deployment process completed successfully!"
