#!/bin/bash

set -euo pipefail

export SECONDS_BETWEEN_UPDATES=10

get_user_config_json() {
    echo "{
        \"simply_plural_token\": \"${SPS_API_TOKEN-null}\",
        \"enable_discord\": ${ENABLE_DISCORD-true},
        \"enable_vrchat\": ${ENABLE_VRCHAT-true},
        \"discord_token\": \"${DISCORD_TOKEN-null}\",
        \"vrchat_username\": \"${VRCHAT_USERNAME-null}\",
        \"vrchat_password\": \"${VRCHAT_PASSWORD-null}\",
        \"vrchat_cookie\": \"${VRCHAT_COOKIE-null}\",
        \"system_name\": \"${SYSTEM_PUBLIC_NAME-null}\",
        \"wait_seconds\": { \"secs\": ${SECONDS_BETWEEN_UPDATES}, \"nanos\": 0 }
    }"
}