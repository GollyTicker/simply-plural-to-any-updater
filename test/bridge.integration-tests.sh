#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

[[ "$VRCHAT_USERNAME" != "" ]]

[[ "$VRCHAT_PASSWORD" != "" ]]

[[ "$DISCORD_STATUS_MESSAGE_TOKEN" != "" ]]

export DISCORD_STATUS_MESSAGE_UPDATER_AVAILABLE=false
ENABLE_DISCORD_STATUS_MESSAGE=false
ENABLE_VRCHAT=false
ENABLE_DISCORD=false

source ./test/source.sh
source ./test/plural_system_to_test.sh

main() {
    stop_updater
    ./dev/tauri-build.sh

    set_system_fronts_set "A"
    start_updater

    start_bridge_frontend_e2e_tests

    stop_updater
    clear_all_fronts
    echo "✅✅✅ Bridge Integration Test ✅✅✅"
}

start_bridge_frontend_e2e_tests() {
    export SP2ANY_BASE_URL="$BASE_URL"
    (cd bridge-frontend && npm run e2e)
}

export BASE_URL="http://localhost:8080"

start_updater() {
    echo "start_updater"
    ./docker/local.start.sh > docker/logs/start.log 2>&1

    setup_test_user

    restart_updaters

    await sp2any-api "Waiting ${SECONDS_BETWEEN_UPDATES}s for next update trigger..."

    echo "Started Updater."
}

stop_updater() {
    echo "stop_updater"
    ./docker/local.stop.sh > docker/logs/stop.log 2>&1
    echo "Stopped Updater."
}
trap stop_updater EXIT

main
