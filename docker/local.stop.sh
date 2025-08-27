#!/bin/bash

set +e

docker compose -f docker/local.compose.yml stop

docker logs sp2any-db > docker/logs/sp2any-db.log 2>&1
docker logs sp2any-api > docker/logs/sp2any-api.log 2>&1
docker logs sp2any-frontend > docker/logs/sp2any-frontend.log 2>&1

docker compose -f docker/local.compose.yml down --volumes --remove-orphans

true
