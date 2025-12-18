#!/bin/bash

set -euo pipefail

export DISCORD_STATUS_MESSAGE_UPDATER_AVAILABLE=false
ENABLE_DISCORD_STATUS_MESSAGE=false

main() {
    mkdir -p target/release_builds/pluralsync-frontend
    tar -xzf target/release_builds/pluralsync-frontend.tar.gz -C target/release_builds/pluralsync-frontend/

    start_backend
    
    echo "Showing logs... Abort with ^C to stop backend."
    docker logs -f pluralsync-api
}

export BASE_URL="http://localhost:8080"

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
