#!/bin/bash
# Helper script to manage LeeAttend Docker environments

set -e

show_usage() {
    echo "🚀 LeeAttend Docker Helper"
    echo "Usage: $0 {dev|test|prod} {command} [args...]"
    echo ""
    echo "Environments:"
    echo "  dev   - Development mode (build target: dev, with hot-reloading)"
    echo "  test  - Testing mode (build target: test, with sccache and tmpfs)"
    echo "  prod  - Production mode (build target: runtime, minimal image)"
    echo ""
    echo "Common Commands:"
    echo "  up -d    - Start services in background"
    echo "  down     - Stop and remove services"
    echo "  build    - Build or rebuild services"
    echo "  ps       - List services"
    echo "  logs -f  - Tail logs"
    echo ""
    echo "Examples:"
    echo "  $0 dev up -d"
    echo "  $0 test build"
    echo "  $0 dev exec apiserver cargo test"
}

if [ $# -lt 2 ]; then
    show_usage
    exit 1
fi

ENV=$1
shift
CMD=$@

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

case "$ENV" in
  dev)
    FILES="-f $PROJECT_ROOT/docker-compose.yml -f $PROJECT_ROOT/docker-compose.dev.yml"
    ;;
  test)
    FILES="-f $PROJECT_ROOT/docker-compose.yml -f $PROJECT_ROOT/docker-compose.test.yml"
    ;;
  prod|base)
    FILES="-f $PROJECT_ROOT/docker-compose.yml"
    ;;
  *)
    show_usage
    exit 1
    ;;
esac

echo "Running: docker compose $FILES $CMD"
docker compose $FILES $CMD
