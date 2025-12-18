#!/bin/bash

set -euo pipefail

export DISCORD_STATUS_MESSAGE_UPDATER_AVAILABLE=true
ENABLE_DISCORD_STATUS_MESSAGE=true
ENABLE_VRCHAT=false
ENABLE_DISCORD=false
ENABLE_WEBSITE=true
ENABLE_TO_PLURALKIT=true

source ./test/source.sh
source ./test/plural_system_to_test.sh

main() {
    stop_updater
    ./steps/12-backend-cargo-build.sh
    clear_all_fronts

    # regression test: Ensure, that restarts don't create duplicate tasks
    start_updater
    sleep 3s
    set_user_config_and_restart
    sleep 5s
    BEFORE_COUNT="$(get_updater_loop_count)"
    set_to_front "$TEST_MEMBER_ID"
    sleep 2s
    AFTER_COUNT="$(get_updater_loop_count)"
    # exactly one update happened in that period
    echo "$BEFORE_COUNT + 1" =? "$AFTER_COUNT"
    [[ "$((BEFORE_COUNT + 1))" == "$AFTER_COUNT" ]]
    echo "✅ no duplicate updater tasks"


    clear_all_fronts
    echo "✅✅✅ Restart Integration Test ✅✅✅"
}

get_updater_loop_count() {
    docker logs pluralsync-api 2>&1 | grep "Waiting for next update trigger..." | wc -l
}

check_updater_loop_continues() {
    echo "check_updater_loop_continues"
    docker logs pluralsync-api 2>&1 | grep -q "Waiting for next update trigger..."
}

export BASE_URL="http://localhost:8080"

start_updater() {
    echo "start_updater"
    ./docker/start.sh local > docker/logs/start.log 2>&1

    setup_test_user

    await pluralsync-api "Waiting for next update trigger..."

    echo "Started startup-test."
}

stop_updater() {
    echo "stop_updater"
    ./docker/stop.sh local > docker/logs/stop.log 2>&1
    echo "Stopped startup-test."
}
trap stop_updater EXIT

main
