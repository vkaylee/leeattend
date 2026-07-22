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

    if [ "${CLOUDFLARE_PURGE_CACHE:-false}" = "true" ] && [ -z "${CLOUDFLARE_ZONE_ID:-}" ]; then
        missing+=("CLOUDFLARE_ZONE_ID (required when CLOUDFLARE_PURGE_CACHE=true)")
    fi
    
    if [ ${#missing[@]} -ne 0 ]; then
        echo "❌ Error: Missing required Cloudflare variables: ${missing[*]}"
        echo "Either provide --env <file> or set them in the environment."
        exit 1
    fi
}

VERIFY_ATTEMPTS="${LEEATTEND_VERIFY_ATTEMPTS:-24}"
VERIFY_INTERVAL_SECONDS="${LEEATTEND_VERIFY_INTERVAL_SECONDS:-5}"

check_required_vars

VERIFY_BASE_URL="${LEEATTEND_DASHBOARD_URL:-https://${CLOUDFLARE_PROJECT_NAME}.pages.dev}"
VERIFY_BASE_URL="${VERIFY_BASE_URL%/}"

if ! [[ "$VERIFY_ATTEMPTS" =~ ^[1-9][0-9]*$ ]]; then
    echo "❌ LEEATTEND_VERIFY_ATTEMPTS must be a positive integer."
    exit 1
fi
if ! [[ "$VERIFY_INTERVAL_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
    echo "❌ LEEATTEND_VERIFY_INTERVAL_SECONDS must be a positive integer."
    exit 1
fi

BUILD_COMMIT=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD)
LEEATTEND_BUILD_ID="${BUILD_COMMIT}-$(date +%s)"
export LEEATTEND_BUILD_ID

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
    -e "LEEATTEND_BUILD_ID=$LEEATTEND_BUILD_ID"
)

# Dynamically add all VITE_* environment variables so Vite can bake them into the build
# compgen might not be available in all shells, but bash usually has it.
for var in $(compgen -v | grep '^VITE_'); do
    COMPOSE_CMD+=("-e" "$var=${!var}")
done

COMPOSE_CMD+=("--entrypoint")

# Step 1: Install dependencies
echo "📦 [1/5] Installing dependencies..."
"${COMPOSE_CMD[@]}" bash webdashboard -c "rm -rf node_modules/.[!.]* node_modules/* && bun install --frozen-lockfile" </dev/null


# Step 2: Build the project
echo "🏗️ [2/5] Building webdashboard..."
"${COMPOSE_CMD[@]}" bash webdashboard -c "NODE_ENV=production bun run build" </dev/null

# Step 3: Deploy to Cloudflare Pages
echo "☁️ [3/5] Deploying to Cloudflare Pages..."
# Ensure the project exists before deploying to prevent interactive prompts
"${COMPOSE_CMD[@]}" bash webdashboard -c "bunx wrangler pages project create \"\$CLOUDFLARE_PROJECT_NAME\" --production-branch \"\${CLOUDFLARE_BRANCH:-main}\" || true" </dev/null
# We use a bash wrapper inside the container to ensure environment variables are expanded correctly
"${COMPOSE_CMD[@]}" bash webdashboard -c "bunx wrangler pages deploy dist --project-name \"\$CLOUDFLARE_PROJECT_NAME\" --branch \"\${CLOUDFLARE_BRANCH:-main}\"" </dev/null

# Step 4: Purging is optional recovery behavior. Hashed assets are immutable,
# while index.html and version.json use no-store and do not need routine purge.
echo ""
if [ "${CLOUDFLARE_PURGE_CACHE:-false}" = "true" ]; then
  echo "🧹 [4/5] Purging Cloudflare cache (explicitly requested)..."
  PURGE_RESPONSE=$(curl --fail-with-body -sS -w "\n%{http_code}" -X POST \
    "https://api.cloudflare.com/client/v4/zones/${CLOUDFLARE_ZONE_ID}/purge_cache" \
    -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{"purge_everything": true}')
  PURGE_STATUS=$(echo "$PURGE_RESPONSE" | tail -1)
  if [ "$PURGE_STATUS" != "200" ]; then
    echo "Cache purge failed with HTTP ${PURGE_STATUS}; aborting release."
    exit 1
  fi
else
  echo "➡️ [4/5] Skipping cache purge; immutable assets and no-store entry points are enabled."
fi

# Step 5: Cloudflare edge propagation is eventually consistent. Poll the
# custom domain until it serves the exact deployment manifest with no-store.
VERIFY_HEADERS=$(mktemp)
VERIFY_BODY=$(mktemp)
trap 'rm -f "$VERIFY_HEADERS" "$VERIFY_BODY"' EXIT

if [ -n "${LEEATTEND_DASHBOARD_URL:-}" ]; then
  echo "🔍 [5/5] Verifying production domain ${VERIFY_BASE_URL} for build ${LEEATTEND_BUILD_ID}..."
else
  echo "🔍 [5/5] LEEATTEND_DASHBOARD_URL is unset; verifying Pages domain ${VERIFY_BASE_URL}..."
fi
for ((attempt = 1; attempt <= VERIFY_ATTEMPTS; attempt++)); do
  VERSION_URL="${VERIFY_BASE_URL}/version.json?_v=$(date +%s)-${attempt}"
  : > "$VERIFY_HEADERS"
  : > "$VERIFY_BODY"

  if curl --fail-with-body -sS -D "$VERIFY_HEADERS" -o "$VERIFY_BODY" "$VERSION_URL" \
    && grep -Fq "\"buildId\":\"${LEEATTEND_BUILD_ID}\"" "$VERIFY_BODY" \
    && grep -Eiq '^cache-control:.*no-store' "$VERIFY_HEADERS"; then
    echo "✅ Verification domain converged on attempt ${attempt}/${VERIFY_ATTEMPTS}."
    break
  fi

  if [ "$attempt" -eq "$VERIFY_ATTEMPTS" ]; then
    echo "Deployment verification failed after ${VERIFY_ATTEMPTS} attempts."
    echo "The custom domain did not serve the expected build with Cache-Control: no-store."
    exit 1
  fi

  echo "⏳ Edge not converged yet (${attempt}/${VERIFY_ATTEMPTS}); retrying in ${VERIFY_INTERVAL_SECONDS}s..."
  sleep "$VERIFY_INTERVAL_SECONDS"
done

echo ""
echo "✅ Deployment process completed successfully!"
