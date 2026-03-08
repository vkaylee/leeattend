#!/usr/bin/env bash
set -euo pipefail

# Cross-build LeeAttendDashboard.exe for Windows from Linux via Docker
#
# Usage:
#   bash build_windows.sh                                          # default: API at 127.0.0.1:3000
#   VITE_API_URL=http://192.168.1.10:3000 bash build_windows.sh    # custom API URL

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

IMAGE_NAME="leeattend-dashboard-win"
OUTPUT_DIR="$SCRIPT_DIR/dist"
EXE_NAME="LeeAttendDashboard.exe"
API_URL="${VITE_API_URL:-http://127.0.0.1:3000/api}"

echo "══════════════════════════════════════════════════"
echo "  Building $EXE_NAME for Windows (cross-compile)"
echo "  API URL: $API_URL"
echo "══════════════════════════════════════════════════"
echo ""

# Build and extract in one step using --output
echo "[1/1] Building and extracting..."
mkdir -p "$OUTPUT_DIR"
podman build \
    -f Dockerfile.windows \
    --build-arg VITE_API_URL="$API_URL" \
    --target output \
    --output "type=local,dest=$OUTPUT_DIR" \
    .

echo ""
echo "══════════════════════════════════════════════════"
echo "  ✅ Build Success!"
echo "  File: $OUTPUT_DIR/$EXE_NAME"
echo "  Size: $(du -h "$OUTPUT_DIR/$EXE_NAME" | cut -f1)"
echo "  API:  $API_URL"
echo "══════════════════════════════════════════════════"
