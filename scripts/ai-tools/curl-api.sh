#!/bin/bash
""${DOCKER_COMPOSE_CMD:-podman-compose}"" -f docker-compose.dev.yml exec apiserver curl -s "$@"
