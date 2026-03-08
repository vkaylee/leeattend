#!/bin/bash
# Usage: ./shell.sh <service_name>
# Example: ./shell.sh apiserver
if [ -z "$1" ]; then
  echo "Usage: ./shell.sh <service_name>"
  echo "Example: ./shell.sh apiserver"
  exit 1
fi
podman-compose -f docker-compose.dev.yml exec "$1" bash
