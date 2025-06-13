#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

source ./test/plural_system_to_test.sh

main() {
    stop_webserver

    start_webserver

    set_system_fronts_set "A"

    check_system_fronts_set "A"

    set_system_fronts_set "B"

    check_system_fronts_set "B"

    stop_webserver

    clear_all_fronts

    echo "✅✅✅ Webserver Integration Test ✅✅✅"
}

check_system_fronts_set() {
    SET="$1"

    HTML="$(curl -s "$WEBSERVER_FRONTING_URL")"

    if [[ "$SET" == "A" ]]; then
        grep '<title>SP-Updater-Test - Fronting Status</title>' <<< "$HTML"
        grep '<div><img src="https://example.com/a" /><p>Annalea 💖 A.</p></div>' <<< "$HTML"
        grep '<div><img src="https://example.com/b" /><p>Borgnen 👍 B.</p></div>' <<< "$HTML"
        grep '<div><img src="" /><p>Daenssa 📶 D.</p></div>' <<< "$HTML"
        [[ "$( grep '<div>' <<< "$HTML" | wc -l )" == "3" ]]
    elif [[ "$SET" == "B" ]]; then
        grep '<title>SP-Updater-Test - Fronting Status</title>' <<< "$HTML"
        grep '<div><img src="" /><p>tešt ▶️ t.</p></div>' <<< "$HTML"
        [[ "$( grep '<div>' <<< "$HTML" | wc -l )" == "1" ]]
    else
        return 1
    fi
}

WEBSERVER_FRONTING_URL="http://0.0.0.0:8000/fronting"

start_webserver() {
    ./cargo-build.sh
    
    set -a; source defaults.env; set +a
    export SPS_API_TOKEN
    export SERVE_API=true
    export SYSTEM_PUBLIC_NAME="SP-Updater-Test"

    ./target/release/sps_status &

    sleep 1s

    echo "Started webserver."
}

stop_webserver() {
    pkill -f sps_status || true
    echo "Stopped webserver."
}

main
