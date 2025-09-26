#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

source ./test/source.sh
source ./test/plural_system_to_test.sh

main() {
    stop_webserver

    ./steps/12-backend-cargo-build.sh

    start_webserver

    set_system_fronts_set "A"

    check_system_fronts_set "A"

    set_system_fronts_set "B"

    check_system_fronts_set "B"

    clear_all_fronts

    echo "✅✅✅ Webserver Integration Test ✅✅✅"
}

check_system_fronts_set() {
    SET="$1"

    HTML="$(curl -s --fail-with-body "$BASE_URL/api/fronting/$USER_ID")"

    if [[ "$SET" == "A" ]]; then
        grep '<title>SP-Updater-Test - Fronting Status</title>' <<< "$HTML"
        grep '<div><img src="https://example.com/a" /><p>Annalea 💖 A.</p></div>' <<< "$HTML"
        grep '<div><img src="https://example.com/b" /><p>Borgnen 👍 B.</p></div>' <<< "$HTML"
        grep '<div><img src="" /><p>Daenssa 📶 D.</p></div>' <<< "$HTML"
        grep '<div><img src="" /><p>Cstm First</p></div>' <<< "$HTML"
        [[ "$( grep '<div>' <<< "$HTML" | wc -l )" == "4" ]]
    elif [[ "$SET" == "B" ]]; then
        grep '<title>SP-Updater-Test - Fronting Status</title>' <<< "$HTML"
        grep '<div><img src="" /><p>tešt ▶️ t. 漢字 クケ 汉字 漢字</p></div>' <<< "$HTML"
        [[ "$( grep '<div>' <<< "$HTML" | wc -l )" == "1" ]]
    else
        return 1
    fi
}

export BASE_URL="http://localhost:8080"

SYSTEM_PUBLIC_NAME="SP-Updater-Test"
ENABLE_DISCORD=false
ENABLE_DISCORD_STATUS_MESSAGE=false
ENABLE_VRCHAT=false
unset DISCORD_STATUS_MESSAGE_TOKEN
unset VRCHAT_USERNAME
unset VRCHAT_PASSWORD
unset VRCHAT_COOKIE

start_webserver() {
    echo "start_webserver"

    ./docker/start.sh local > docker/logs/start.log 2>&1

    setup_test_user

    echo "Started webserver."
}

stop_webserver() {
    echo "stop_webserver"
    ./docker/stop.sh local > docker/logs/start.log 2>&1
    echo "Stopped webserver."
}
trap stop_webserver EXIT

main
