#!/bin/bash

set +e


docker logs sp2any-db > docker/logs/sp2any-db.log 2>&1
docker logs sp2any-api > docker/logs/sp2any-api.log 2>&1
docker logs sp2any-entrypoint > docker/logs/sp2any-entrypoint.log 2>&1

docker compose -f docker/docker.compose.yml stop

docker compose -f docker/docker.compose.yml down --volumes --remove-orphans

true
