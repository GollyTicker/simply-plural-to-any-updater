#!/bin/bash

set +e

export PLURALSYNC_STAGE="$1"

docker logs pluralsync-db > docker/logs/pluralsync-db.log 2>&1
docker logs pluralsync-api > docker/logs/pluralsync-api.log 2>&1
docker logs pluralsync-entrypoint > docker/logs/pluralsync-entrypoint.log 2>&1
docker logs pluralsync-global-manager > docker/logs/pluralsync-global-manager.log 2>&1

docker compose -f docker/docker.compose.yml stop

docker compose -f docker/docker.compose.yml down --volumes --remove-orphans

true
