#!/bin/bash

set +e

export PATH_TO_CONFIG_JSON="${PATH_TO_CONFIG_JSON-./does-not-exist}"

docker compose -f docker/local.compose.yml stop

docker logs sp2any-webserver > docker/logs/sp2any-webserver.log

docker logs sp2any-db > docker/logs/sp2any-db.log

docker compose -f docker/local.compose.yml down --volumes --remove-orphans

true
