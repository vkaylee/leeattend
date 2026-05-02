#!/bin/bash
""${DOCKER_COMPOSE_CMD:-podman-compose}"" -f docker-compose.dev.yml exec db psql -U admin -d leeattend "$@"
