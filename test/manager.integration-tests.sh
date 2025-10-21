#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

FUNCTIONAL_DISCORD_STATUS_MESSAGE_TOKEN="$DISCORD_STATUS_MESSAGE_TOKEN"
FUNCTIONAL_PLURALKIT_TOKEN="$PLURALKIT_TOKEN"
FUNCTIONAL_SPS_API_TOKEN="$SPS_API_TOKEN"

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


    setup_sp_rest_failure
    start_updater
    check_updater_failure
    check_logs_contain "Retrying in 3600 seconds..."
    check_updater "DiscordStatusMessage" "Running"
    check_updater "ToPluralKit" "Running"
    reset_changed_variables
    echo "✅ simply plural rest failure"
    

    stop_updater
    setup_discord_status_message_not_available
    start_updater
    check_updater_has_no_errors
    check_updater_loop_continues
    check_updater "ToPluralKit" "Running"
    check_missing "DiscordStatusMessage"
    reset_changed_variables
    echo "✅ discord status message failure"


    setup_topluralkit_only
    start_updater
    check_updater_has_no_errors
    check_updater_loop_continues
    check_updater "DiscordStatusMessage" "Disabled"
    check_updater "ToPluralKit" "Running"
    reset_changed_variables
    echo "✅ discord status message disabled"


    setup_discord_status_message_only
    start_updater
    check_updater_has_no_errors
    check_updater_loop_continues
    check_updater "DiscordStatusMessage" "Running"
    check_updater "ToPluralKit" "Disabled"
    reset_changed_variables
    echo "✅ topluralkit disabled"


    setup_topluralkit_misconfigured
    start_updater
    check_updater_failure
    check_updater_loop_continues
    check_updater "DiscordStatusMessage" "Running"
    get_updater_statuses | jq -r ".ToPluralKit" | grep -q "Error"
    reset_changed_variables
    echo "✅ topluralkit failed"


    clear_all_fronts
    echo "✅✅✅ Manager Integration Test ✅✅✅"
}


check_logs_contain() {
    echo "check_logs_contain '$1'"
    docker logs sp2any-api 2>&1 | grep -q "$1"
}

check_updater_loop_continues() {
    echo "check_updater_loop_continues"
    docker logs sp2any-api 2>&1 | grep -q "Waiting for next update trigger..."
}

check_updater_has_no_errors() {
    echo "check_updater_has_no_errors"
    [[ "$( docker logs sp2any-api 2>&1 | grep -i "Error" | wc -l )" == "0" ]]
}

check_updater_failure() {
    echo "check_updater_failure"
    [[ "$( docker logs sp2any-api 2>&1 | grep -i "Error" | wc -l )" != "0" ]]
}

check_updater() {
    PLATFORM="$1"
    STATUS="$2"
    echo "Check $PLATFORM is $STATUS ?"
    RES="$( get_updater_statuses | jq ".$PLATFORM == \"$STATUS\"" )"
    [[ "$RES" == "true" ]]    
}

check_missing() {
    MESSAGE="$1"
    echo "Check missing: '$MESSAGE'"
    set +e
    N_LINES="$( docker logs sp2any-api 2>&1 | grep "$MESSAGE" | wc -l )"
    set -e
    [[ "$N_LINES" == "0" ]]
}

setup_sp_rest_failure() {
    echo "setup_sp_rest_failure"
    SPS_API_TOKEN="invalid"
}

setup_topluralkit_only() {
    echo "setup_topluralkit_only"
    DISCORD_STATUS_MESSAGE_TOKEN="invalid"
    ENABLE_DISCORD_STATUS_MESSAGE=false
}

setup_discord_status_message_only() {
    echo "setup_discord_status_message_only"
    ENABLE_TO_PLURALKIT=false
    PLURALKIT_TOKEN=""
}

setup_discord_status_message_not_available() {
    echo "setup_discord_status_message_not_available"
    unset DISCORD_STATUS_MESSAGE_UPDATER_AVAILABLE
}

setup_topluralkit_misconfigured() {
    echo "setup_topluralkit_misconfigured"
    PLURALKIT_TOKEN=""
}


reset_changed_variables() {
    echo "reset_changed_variables"
    DISCORD_STATUS_MESSAGE_TOKEN="$FUNCTIONAL_DISCORD_STATUS_MESSAGE_TOKEN"
    PLURALKIT_TOKEN="$FUNCTIONAL_PLURALKIT_TOKEN"
    SPS_API_TOKEN="$FUNCTIONAL_SPS_API_TOKEN"
    export DISCORD_STATUS_MESSAGE_UPDATER_AVAILABLE=true
    ENABLE_DISCORD_STATUS_MESSAGE=true
    ENABLE_TO_PLURALKIT=true
}


export BASE_URL="http://localhost:8080"

start_updater() {
    echo "start_updater"
    ./docker/start.sh local > docker/logs/start.log 2>&1

    setup_test_user

    await sp2any-api "client authentication sent."
    
    sleep 3s

    echo "Started startup-test."
}

stop_updater() {
    echo "stop_updater"
    ./docker/stop.sh local > docker/logs/stop.log 2>&1
    echo "Stopped startup-test."
}
trap stop_updater EXIT

main
