#!/usr/bin/env bash
# ==============================================================================
# Production Image Builder — LeeAttend
#
# Builds optimized container images for apiserver and agent-main.
# Handles apiserver's sqlx compile-time DB validation automatically.
#
# Usage:
#   ./scripts/build-prod.sh                          # Build all, tag :latest
#   ./scripts/build-prod.sh --service apiserver       # Build one service
#   ./scripts/build-prod.sh --tag v1.2.3 --push       # Build all + push
#   ./scripts/build-prod.sh --tag $(git rev-parse --short HEAD)
#   ./scripts/build-prod.sh --registry ghcr.io/user   # Custom registry
#   ./scripts/build-prod.sh --dry-run                 # Show what would run
# ==============================================================================
set -euo pipefail

# ── Color output ─────────────────────────────────────────────────────
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

if [[ -n "${NO_COLOR:-}" ]]; then
    GREEN='' RED='' YELLOW='' BLUE='' CYAN='' BOLD='' NC=''
fi

# ── Globals ──────────────────────────────────────────────────────────
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REGISTRY="${REGISTRY:-docker.io/vleedev}"
TAG="latest"
IMAGE_OVERRIDE=""
SERVICES=()
PUSH=false
DRY_RUN=false
ALSO_TAG_LATEST=false
ENGINE=""
ENGINE=""

# All buildable services
ALL_SERVICES=("apiserver" "agent-main")

# Services requiring sqlx offline cache
SERVICES_NEED_DB=("apiserver")

# ── Helpers ──────────────────────────────────────────────────────────
log()   { echo -e "${BLUE}[build]${NC} $*"; }
ok()    { echo -e "${GREEN}  ✔${NC} $*"; }
warn()  { echo -e "${YELLOW}  ⚠${NC} $*" >&2; }
err()   { echo -e "${RED}  ✖${NC} $*" >&2; }
dim()   { echo -e "${DIM:-}       $*${NC}"; }

die() { err "$@"; exit 1; }

# ── Engine detection ─────────────────────────────────────────────────
detect_engine() {
    if command -v podman &>/dev/null; then
        ENGINE="podman"
    elif command -v docker &>/dev/null; then
        ENGINE="docker"
    else
        die "Neither podman nor docker found. Install one to continue."
    fi
    ok "Engine: ${BOLD}$ENGINE${NC}"
}

# ── CLI Parser ───────────────────────────────────────────────────────
usage() {
    echo -e "${BOLD}Usage:${NC} ./scripts/build-prod.sh [OPTIONS]

${BOLD}Options:${NC}
  --service NAME     (Required) Build a specific service (apiserver, agent-main)
  --tag TAG          Image tag (default: latest)
  --image NAME       Override full image name (e.g. myreg.io/myapp)
                     If specified, --registry and the 'leeattend-' prefix are ignored.
  --registry URL     Registry prefix (default: \$REGISTRY or docker.io/vleedev)
  --push             Push images after building
  --also-latest      Also tag as :latest when using a specific tag
  --dry-run          Show commands without executing
  -h, --help         Show this help

${BOLD}Examples:${NC}
  ./scripts/build-prod.sh --service apiserver --tag v1.2.3 --push
  ./scripts/build-prod.sh --service apiserver --image myreg.io/my-custom-api --push
  ./scripts/build-prod.sh --service agent-main --tag \$(git rev-parse --short HEAD) --push --also-latest

${BOLD}Environment:${NC}
  REGISTRY           Override default registry prefix
  APISERVER_IMAGE    Override full image name for apiserver
  AGENT_IMAGE        Override full image name for agent-main
  BUILD_DB_PORT      Override Postgres port for apiserver build (auto-assigned if empty)
"
    exit 0
}

parse_args() {
    if [[ $# -eq 0 ]]; then
        usage
    fi
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --service)
                [[ -n "${2:-}" ]] || die "--service requires a value"
                if [[ ${#SERVICES[@]} -gt 0 ]]; then
                    die "Error: You can only specify one service at a time."
                fi
                SERVICES+=("$2"); shift 2 ;;
            --tag)
                [[ -n "${2:-}" ]] || die "--tag requires a value"
                TAG="$2"; shift 2 ;;
            --image)
                [[ -n "${2:-}" ]] || die "--image requires a value"
                IMAGE_OVERRIDE="$2"; shift 2 ;;
            --registry)
                [[ -n "${2:-}" ]] || die "--registry requires a value"
                REGISTRY="$2"; shift 2 ;;
            --push)
                PUSH=true; shift ;;
            --also-latest)
                ALSO_TAG_LATEST=true; shift ;;
            --dry-run)
                DRY_RUN=true; shift ;;
            -h|--help)
                usage ;;
            *)
                die "Unknown argument: $1. Use --help for usage." ;;
        esac
    done

    # Ensure exactly one service is specified
    if [[ ${#SERVICES[@]} -eq 0 ]]; then
        die "Error: You must specify a service using --service NAME. Valid services: ${ALL_SERVICES[*]}"
    fi

    # Validate service names
    for svc in "${SERVICES[@]}"; do
        local valid=false
        for known in "${ALL_SERVICES[@]}"; do
            [[ "$svc" == "$known" ]] && valid=true
        done
        $valid || die "Unknown service: $svc. Valid: ${ALL_SERVICES[*]}"
    done
}

# ── Dry-run wrapper ──────────────────────────────────────────────────
run() {
    if $DRY_RUN; then
        dim "[dry-run] $*"
        return 0
    fi
    "$@"
}

# ── Find a free port ─────────────────────────────────────────────────
find_free_port() {
    python3 -c "import socket; s=socket.socket(); s.bind(('',0)); print(s.getsockname()[1]); s.close()" 2>/dev/null \
        || shuf -i 10000-60000 -n 1
}

# ── Prepare SQLx Cache ───────────────────────────────────────────────
needs_sqlx_cache() {
    local svc="$1"
    for db_svc in "${SERVICES_NEED_DB[@]}"; do
        [[ "$svc" == "$db_svc" ]] && return 0
    done
    return 1
}

prepare_sqlx_cache() {
    log "Preparing SQLx offline cache using existing orchestrator..."
    if $DRY_RUN; then
        ok "SQLx cache would be prepared (dry-run)"
        return
    fi

    # Delegate to the Python orchestrator via _sqlx.sh
    # It automatically handles the DB container lifecycle
    "${PROJECT_ROOT}/scripts/_sqlx.sh" prepare --workspace
    ok "SQLx cache prepared successfully"
}

# ── Image naming ─────────────────────────────────────────────────────
image_name() {
    local svc="$1"
    local tag="${2:-$TAG}"
    
    # Check for service-specific image override via environment variables
    local env_image=""
    if [[ "$svc" == "apiserver" ]]; then
        env_image="${APISERVER_IMAGE:-}"
    elif [[ "$svc" == "agent-main" ]]; then
        env_image="${AGENT_IMAGE:-}"
    fi

    if [[ -n "$IMAGE_OVERRIDE" ]]; then
        # CLI override takes highest precedence
        env_image="$IMAGE_OVERRIDE"
    fi

    if [[ -n "$env_image" ]]; then
        # If the override already has a tag (contains a colon), use it as is for the main tag,
        # otherwise append the specified tag.
        if [[ "$env_image" == *:* ]]; then
            echo "$env_image"
        else
            echo "${env_image}:${tag}"
        fi
    else
        echo "${REGISTRY}/leeattend-${svc}:${tag}"
    fi
}

# ── Build a single service ───────────────────────────────────────────
build_service() {
    local svc="$1"
    local context="${PROJECT_ROOT}/${svc}"
    local full_image
    full_image="$(image_name "$svc")"

    echo ""
    log "━━━ Building ${BOLD}${CYAN}$svc${NC} → ${DIM:-}$full_image${NC} ━━━"

    # Validate context
    [[ -f "${context}/Dockerfile" ]] || die "No Dockerfile at ${context}/Dockerfile"

    # Build args
    local build_args=()
    build_args+=(build)
    build_args+=(--target runtime)
    build_args+=(-t "$full_image")

    build_args+=(-f "${context}/Dockerfile")
    build_args+=("$PROJECT_ROOT")

    dim "$ENGINE ${build_args[*]}"
    run $ENGINE "${build_args[@]}"
    ok "Built: $full_image"

    # Also tag as :latest if requested
    if $ALSO_TAG_LATEST && [[ "$TAG" != "latest" ]]; then
        local latest_image
        latest_image="$(image_name "$svc" "latest")"
        run $ENGINE tag "$full_image" "$latest_image"
        ok "Tagged: $latest_image"
    fi
}

# ── Push images ──────────────────────────────────────────────────────
push_service() {
    local svc="$1"
    local full_image
    full_image="$(image_name "$svc")"

    log "Pushing ${BOLD}$full_image${NC}..."
    run $ENGINE push "$full_image"
    ok "Pushed: $full_image"

    if $ALSO_TAG_LATEST && [[ "$TAG" != "latest" ]]; then
        local latest_image
        latest_image="$(image_name "$svc" "latest")"
        run $ENGINE push "$latest_image"
        ok "Pushed: $latest_image"
    fi
}

# ── Main ─────────────────────────────────────────────────────────────
main() {
    parse_args "$@"
    detect_engine

    echo ""
    echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}║           LeeAttend — Production Image Builder             ║${NC}"
    echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "  ${CYAN}Registry :${NC} $REGISTRY"
    echo -e "  ${CYAN}Tag      :${NC} $TAG"
    echo -e "  ${CYAN}Services :${NC} ${SERVICES[*]}"
    echo -e "  ${CYAN}Push     :${NC} $PUSH"
    echo -e "  ${CYAN}Dry Run  :${NC} $DRY_RUN"
    $ALSO_TAG_LATEST && echo -e "  ${CYAN}Also     :${NC} :latest"
    echo ""

    # Determine if we need to prepare SQLx cache
    local need_sqlx=false
    for svc in "${SERVICES[@]}"; do
        needs_sqlx_cache "$svc" && need_sqlx=true
    done

    # Prepare SQLx cache if needed
    if $need_sqlx; then
        prepare_sqlx_cache
    fi

    # Build phase
    local start_time=$SECONDS
    for svc in "${SERVICES[@]}"; do
        build_service "$svc"
    done
    local build_time=$(( SECONDS - start_time ))

    # Push phase
    if $PUSH; then
        echo ""
        log "━━━ Pushing images ━━━"
        for svc in "${SERVICES[@]}"; do
            push_service "$svc"
        done
    fi

    # Summary
    echo ""
    echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}║                    Build Complete ✅                        ║${NC}"
    echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "  ${GREEN}Duration:${NC} ${build_time}s"
    echo -e "  ${GREEN}Images:${NC}"
    for svc in "${SERVICES[@]}"; do
        echo -e "    → $(image_name "$svc")"
        $ALSO_TAG_LATEST && [[ "$TAG" != "latest" ]] && echo -e "    → $(image_name "$svc" "latest")"
    done
    echo ""

    if ! $PUSH; then
        echo -e "  ${DIM:-}Tip: Add --push to push images to the registry.${NC}"
    fi
}

main "$@"
