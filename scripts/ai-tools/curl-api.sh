#!/bin/bash
podman-compose -f docker-compose.dev.yml exec apiserver curl -s "$@"
