#!/bin/bash
docker compose -f docker-compose.dev.yml exec db psql -U admin -d leeattend "$@"
