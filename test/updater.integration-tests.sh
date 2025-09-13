#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

[[ "$VRCHAT_USERNAME" != "" ]]

[[ "$VRCHAT_PASSWORD" != "" ]]

[[ "$VRCHAT_COOKIE" != "" ]]

[[ "$DISCORD_TOKEN" != "" ]]

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


    stop_updater

    setup_sp_rest_failure

    start_updater

    check_updater_failure_and_loop_continues


    stop_updater

    clear_all_fronts

    echo "✅✅✅ Updater Integration Test ✅✅✅"
}


check_system_fronts_set() {
    SET="$1"

    if [[ "$SET" == "A" ]]; then
        check_vrc_status_string_equals "F˸Ann‚Bor‚Dae‚Cst"
        check_discord_status_string_equals "F: Annalea 💖 A., Borgn B., Daenssa 📶 D., Cstm First"
    elif [[ "$SET" == "B" ]]; then
        check_vrc_status_string_equals "F˸ tš漢ク汉漢"
        check_discord_status_string_equals "F: tš▶️漢ク汉漢"
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
        -H "Authorization: $DISCORD_TOKEN"
    )"

    STATUS="$( echo "$RESPONSE" | jq -r .custom_status.text )"

    echo "Discord Status Check: '$STATUS' =? '$EXPECTED'"

    [[ "$STATUS" == "$EXPECTED" ]]
}


setup_sp_rest_failure() {
    FUNCTIONAL_SPS_API_TOKEN="$SPS_API_TOKEN"
    SPS_API_TOKEN="invalid"
}

check_updater_failure_and_loop_continues() {
    SPS_API_TOKEN="$FUNCTIONAL_SPS_API_TOKEN"
    grep -q "Error: " .log
    grep -q "Waiting ${SECONDS_BETWEEN_UPDATES}s for next update trigger..." .log
}



start_updater() {
    write_env_vars_to_config_json

    (./target/release/sp2any --no-gui --config "$CONFIG_FILE" 2>&1 | tee .log | sed 's/^/sp2any | /' ) &

    sleep 5s

    echo "Started Updater."
}

stop_updater() {
    pkill -f sp2any || true
    echo "Stopped Updater."
}

main
