#!/bin/bash

set +e

docker compose -f docker/local.compose.yml stop

docker logs sp2any-webserver > docker/logs/sp2any-webserver.log 2>&1

docker logs sp2any-db > docker/logs/sp2any-db.log 2>&1

docker compose -f docker/local.compose.yml down --volumes --remove-orphans

true
