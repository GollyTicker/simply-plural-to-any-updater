#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

[[ "$VRCHAT_USERNAME" != "" ]]

[[ "$VRCHAT_PASSWORD" != "" ]]

[[ "$VRCHAT_COOKIE" != "" ]]

source ./test/plural_system_to_test.sh

SECONDS_BETWEEN_UPDATES=10

main() {
    stop_vrc_updater

    set_system_fronts_set "A"
    
    start_vrc_updater

    check_system_fronts_set "A"
    
    set_system_fronts_set "B"

    sleep "$SECONDS_BETWEEN_UPDATES"s

    check_system_fronts_set "B"

    stop_vrc_updater

    clear_all_fronts

    echo "✅✅✅ VRC Updater Integration Test ✅✅✅"
}


check_system_fronts_set() {
    SET="$1"

    if [[ "$SET" == "A" ]]; then
        check_vrc_status_string_equals "F˸Ann‚Bor‚Dae"
    elif [[ "$SET" == "B" ]]; then
        check_vrc_status_string_equals "F˸ tešt t․"
    else
        return 1
    fi
}


check_vrc_status_string_equals() {
    EXPECTED="$1"

    RESPONSE="$(curl -s "https://api.vrchat.cloud/api/1/auth/user" \
        --cookie "$VRCHAT_COOKIE" \
        -u "$VRCHAT_USERNAME:$VRCHAT_PASSWORD" \
        -H "User-Agent: SimplyPluralToVRChatUpdaterTest/0.1.0 does-not-exist-792374@gmail.com"
    )"

    STATUS="$( echo "$RESPONSE" | jq -r .statusDescription)"

    echo "VRC Status Check: '$STATUS' =? '$EXPECTED'"

    [[ "$STATUS" == "$EXPECTED" ]]
}


start_vrc_updater() {
    cargo build --release

    rm -rf vrcupdater.env || true

    echo "
SPS_API_TOKEN=\"$SPS_API_TOKEN\"
VRCHAT_USERNAME=\"$VRCHAT_USERNAME\"
VRCHAT_PASSWORD=\"$VRCHAT_PASSWORD\"
VRCHAT_COOKIE=\"$VRCHAT_COOKIE\"
SECONDS_BETWEEN_UPDATES=\"$SECONDS_BETWEEN_UPDATES\"
    " >> vrcupdater.env

    ./target/release/sps_status &

    sleep 5s

    echo "Started VRC Updater."
}

stop_vrc_updater() {
    pkill -f sps_status || true
    echo "Stopped VRC Updater."
}

main
