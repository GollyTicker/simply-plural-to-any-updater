#!/bin/bash

set -euo pipefail

export DISCORD_STATUS_MESSAGE_UPDATER_AVAILABLE=false
ENABLE_DISCORD_STATUS_MESSAGE=false

export SP2ANY_BASE_URL="http://localhost:23327"

main() {
    rm -rf output/sp2any-frontend || true
    mkdir -p output/sp2any-frontend
    tar -xzf output/sp2any-frontend.tar.gz -C output/sp2any-frontend/
    sed -i "s|__SP2ANY_BASE_URL__|${SP2ANY_BASE_URL}|g" output/sp2any-frontend/assets/*.js
    echo "Frontend substitution: SP2ANY_BASE_URL=$SP2ANY_BASE_URL"

    start_backend
    
    echo "Showing logs... Abort with ^C to stop backend."
    docker logs -f sp2any-api
}

start_backend() {
    echo "start_backend"
    ./docker/start.sh local-release # > docker/logs/start.log 2>&1
    echo "Started Backend."
}

stop_backend() {
    echo "stop_backend"
    ./docker/stop.sh local-release > docker/logs/stop.log 2>&1
    echo "Stop Backend."
}
trap stop_backend EXIT

main
