#!/bin/bash

set -euo pipefail

[[ "$SPS_API_TOKEN" != "" ]]

[[ "$SPS_API_WRITE_TOKEN" != "" ]]

source ./test/source.sh
source ./test/plural_system_to_test.sh

main() {
    stop_webserver

    ./release/cargo-build.sh

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

WEBSERVER_FRONTING_URL="http://0.0.0.0:8000/fronting"

start_webserver() {
    set -a; source release/config/server.defaults.env; set +a
    export SYSTEM_PUBLIC_NAME="SP-Updater-Test"
    write_env_vars_to_config_json

    (./target/release/sp2any --webserver --config "$CONFIG_FILE" 2>&1 | tee .log | sed 's/^/sp2any | /' ) &

    sleep 1s

    echo "Started webserver."
}

stop_webserver() {
    pkill -f sp2any || true
    echo "Stopped webserver."
}

main
