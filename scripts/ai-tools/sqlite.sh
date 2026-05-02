#!/bin/bash
""${DOCKER_COMPOSE_CMD:-podman-compose}"" -f docker-compose.dev.yml exec agent-main sqlite3 /app/data/agent.db "$@"
