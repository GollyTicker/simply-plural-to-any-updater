#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

[[ "$VRCHAT_USERNAME" != "" ]]

[[ "$VRCHAT_PASSWORD" != "" ]]

[[ "$VRCHAT_COOKIE" != "" ]]

[[ "$DISCORD_STATUS_MESSAGE_TOKEN" != "" ]]

export DISCORD_STATUS_MESSAGE_UPDATER_AVAILABLE=true
ENABLE_DISCORD_STATUS_MESSAGE=true
ENABLE_VRCHAT=true
ENABLE_DISCORD=false

source ./test/source.sh
source ./test/plural_system_to_test.sh

main() {
    stop_updater
    ./release/cargo-build.sh


    set_system_fronts_set "A"
    start_updater
    check_system_fronts_set "A"
    set_system_fronts_set "B"
    sleep "$SECONDS_BETWEEN_UPDATES"s
    check_system_fronts_set "B"

    echo "Sleeping now... now you can do stuff!"
    sleep 500s


    stop_updater
    clear_all_fronts
    echo "✅✅✅ Updater Integration Test ✅✅✅"
}


check_system_fronts_set() {
    SET="$1"
    echo "check_system_fronts_set '$SET'"

    if [[ "$SET" == "A" ]]; then
        check_vrc_status_string_equals "F˸Ann‚Bor‚Dae‚Cst"
        check_discord_status_string_equals "F: Annalea 💖 A., Borgn B., Daenssa 📶 D., Cstm First"
    elif [[ "$SET" == "B" ]]; then
        check_vrc_status_string_equals "F˸ tešt t․"
        check_discord_status_string_equals "F: tešt ▶️ t."
    else
        return 1
    fi
}


check_vrc_status_string_equals() {
    EXPECTED="$1"

    RESPONSE="$(curl -s "https://api.vrchat.cloud/api/1/auth/user" \
        --cookie "$VRCHAT_COOKIE" \
        -u "$VRCHAT_USERNAME:$VRCHAT_PASSWORD" \
        -H "User-Agent: SP2Any/0.1.0 does-not-exist-792374@gmail.com"
    )"

    STATUS="$( echo "$RESPONSE" | jq -r .statusDescription)"

    echo "VRC Status Check: '$STATUS' =? '$EXPECTED'"

    [[ "$STATUS" == "$EXPECTED" ]]
}

check_discord_status_string_equals() {
    EXPECTED="$1"

    RESPONSE="$(curl -s \
        "https://discord.com/api/v10/users/@me/settings" \
        -H "Authorization: $DISCORD_STATUS_MESSAGE_TOKEN"
    )"

    STATUS="$( echo "$RESPONSE" | jq -r .custom_status.text )"

    echo "Discord Status Check: '$STATUS' =? '$EXPECTED'"

    [[ "$STATUS" == "$EXPECTED" ]]
}


export BASE_URL="http://localhost:8000"

start_updater() {
    echo "start_updater"
    ./docker/local.start.sh > docker/logs/start.log 2>&1

    setup_test_user

    restart_updaters

    await sp2any-webserver "Waiting ${SECONDS_BETWEEN_UPDATES}s for next update trigger..."

    echo "Started Updater."
}

stop_updater() {
    echo "stop_updater"
    ./docker/local.stop.sh > docker/logs/stop.log 2>&1
    echo "Stopped Updater."
}
trap stop_updater EXIT

main
