#!/bin/bash
""${DOCKER_COMPOSE_CMD:-podman-compose}"" -f docker-compose.dev.yml exec redis redis-cli -a ${REDIS_PASSWORD:-dev_password} "$@"
