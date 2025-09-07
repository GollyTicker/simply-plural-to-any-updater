#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

export DISCORD_STATUS_MESSAGE_UPDATER_AVAILABLE=false
ENABLE_DISCORD_STATUS_MESSAGE=false
ENABLE_VRCHAT=false
ENABLE_DISCORD=true

source ./test/source.sh
source ./test/plural_system_to_test.sh

main() {
    ./release/cargo-build.sh
    set_system_fronts_set "A"
    start_backend
    
    echo "Showing logs... Abort with ^C to stop backend."
    docker logs -f sp2any-api
}

export BASE_URL="http://localhost:8080"

start_backend() {
    echo "start_backend"
    ./docker/local.start.sh > docker/logs/start.log 2>&1

    setup_test_user

    restart_updaters

    await sp2any-api "Waiting ${SECONDS_BETWEEN_UPDATES}s for next update trigger..."

    echo "Started Backend."
}

stop_backend() {
    echo "stop_backend"
    ./docker/local.stop.sh > docker/logs/stop.log 2>&1
    echo "Stop Backend."
}
trap stop_backend EXIT

main
