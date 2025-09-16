#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

[[ "$VRCHAT_USERNAME" != "" ]]

[[ "$VRCHAT_PASSWORD" != "" ]]

[[ "$DISCORD_STATUS_MESSAGE_TOKEN" != "" ]]

export DISCORD_STATUS_MESSAGE_UPDATER_AVAILABLE=true
ENABLE_DISCORD_STATUS_MESSAGE=true
ENABLE_VRCHAT=true
ENABLE_DISCORD=false

source ./test/source.sh
source ./test/plural_system_to_test.sh
set -a; source ./test/ensure-vrchat-cookie-available.dev.sh --automated ; set +a

main() {
    stop_updater
    ./steps/12-backend-cargo-build.sh


    set_system_fronts_set "A"
    start_updater

    # echo "Sleeping. Now you can do stuff!"
    # sleep 2h

    check_system_fronts_set "A"
    set_system_fronts_set "B"
    sleep "$SECONDS_BETWEEN_UPDATES"s
    check_system_fronts_set "B"


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
        check_vrc_status_string_equals "F˸ tš"
        check_discord_status_string_equals "F: tš▶️漢ク汉漢"
    else
        return 1
    fi
}


check_vrc_status_string_equals() {
    EXPECTED="$1"

    VRCHAT_COOKIE_STR="$(echo -n "$VRCHAT_COOKIE" | base64 --decode | jq -r .[0].raw_cookie)"

    RESPONSE="$(curl -s "https://api.vrchat.cloud/api/1/auth/user" \
        --cookie "$VRCHAT_COOKIE_STR" \
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
