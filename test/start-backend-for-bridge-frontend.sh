#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

[[ "$VRCHAT_USERNAME" != "" ]]

[[ "$VRCHAT_PASSWORD" != "" ]]

export DISCORD_STATUS_MESSAGE_UPDATER_AVAILABLE=false
ENABLE_DISCORD_STATUS_MESSAGE=false
ENABLE_VRCHAT=true
ENABLE_DISCORD=true
ENABLE_WEBSITE=true

source ./test/source.sh
source ./test/plural_system_to_test.sh

main() {
    ./steps/12-backend-cargo-build.sh
    ./steps/17-frontend-npm-build.sh
    set_system_fronts_set "A"
    start_backend
    
    echo "Showing logs... Abort with ^C to stop backend."
    docker logs -f sp2any-api
}

export BASE_URL="http://localhost:8080"

start_backend() {
    echo "start_backend"
    ./docker/start.sh local > docker/logs/start.log 2>&1

    setup_test_user

    await sp2any-api "Waiting ${SECONDS_BETWEEN_UPDATES}s for next update trigger..."

    echo "Started Backend."
}

stop_backend() {
    echo "stop_backend"
    ./docker/stop.sh local > docker/logs/stop.log 2>&1
    echo "Stop Backend."
}
trap stop_backend EXIT

main
