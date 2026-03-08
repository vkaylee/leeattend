#!/bin/bash
podman-compose -f docker-compose.dev.yml exec agent-main sqlite3 /app/data/agent.db "$@"
