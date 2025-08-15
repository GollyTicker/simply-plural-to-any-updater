#!/bin/bash

set -euo pipefail


export JWT_APPLICATION_SECRET="some-jwt-secret"
export APPLICATION_USER_SECRETS="some-app-user-secret"

export SECONDS_BETWEEN_UPDATES=10
export SYSTEM_PUBLIC_NAME=ayake-test

get_user_config_json() {
    echo "{
        \"simply_plural_token\": { \"secret\": \"${SPS_API_TOKEN}\" },
        \"enable_discord\": ${ENABLE_DISCORD},
        \"enable_vrchat\": ${ENABLE_VRCHAT},
        \"discord_token\":  { \"secret\": \"${DISCORD_TOKEN}\" },
        \"vrchat_username\":  { \"secret\": \"${VRCHAT_USERNAME}\" },
        \"vrchat_password\":  { \"secret\": \"${VRCHAT_PASSWORD}\" },
        \"vrchat_cookie\":  { \"secret\": \"${VRCHAT_COOKIE}\" },
        \"system_name\": \"${SYSTEM_PUBLIC_NAME-null}\",
        \"wait_seconds\": ${SECONDS_BETWEEN_UPDATES-null}
    }"
}
export -f get_user_config_json


setup_test_user() {
    echo "Creating user ..."
    JSON="{
        \"email\": { \"inner\": \"test@example.com\" },
        \"password\": { \"inner\": \"m?3yp%&wdS+\" }
    }"
    curl -s --fail-with-body \
        -H "Content-Type: application/json" \
        -d "$JSON" \
        "$BASE_URL/api/user/register"

    echo "Logging in ..."
    JWT_JSON="$(
        curl -s --fail-with-body \
            -H "Content-Type: application/json" \
            -d "$JSON" \
            "$BASE_URL/api/user/login"
    )"

    JWT="$(echo "$JWT_JSON" | jq -r .inner)"
    export USER_ID="$(echo "$JWT" | cut -d'.' -f2 | base64 --decode | jq -r .sub)"
    echo "Received Jwt: $JWT"
    echo "User ID: $USER_ID"

    echo "Setting config ..."
    JSON="$(get_user_config_json)"
    curl -s --fail-with-body \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $JWT" \
        -d "$JSON" \
        "$BASE_URL/api/user/config"

    # set -x
    # start updaters
    # check status

    # set +x
    # return 1

    echo "Test user setup complete."
}

