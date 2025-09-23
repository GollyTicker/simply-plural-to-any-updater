#!/bin/bash

set -euo pipefail


export JWT_APPLICATION_SECRET="some-jwt-secret"
export APPLICATION_USER_SECRETS="some-app-user-secret"

export SECONDS_BETWEEN_UPDATES=10
export SYSTEM_PUBLIC_NAME=ayake-test

source docker/source.sh # await

get_user_config_json() {

    if [ -v DISCORD_STATUS_MESSAGE_TOKEN ] ; then 
        DISCORD_STATUS_MESSAGE_TOKEN_LINE="\"discord_status_message_token\": { \"secret\": \"${DISCORD_STATUS_MESSAGE_TOKEN}\" },"
    else
        DISCORD_STATUS_MESSAGE_TOKEN_LINE=""
    fi

    if [ -v SPS_API_TOKEN ] ; then
        SIMPLY_PLURAL_TOKEN_LINE="\"simply_plural_token\": { \"secret\": \"${SPS_API_TOKEN}\" },"
    else
        SIMPLY_PLURAL_TOKEN_LINE=""
    fi

    if [ -v VRCHAT_USERNAME ] ; then
        VRCHAT_USERNAME_LINE="\"vrchat_username\": { \"secret\": \"${VRCHAT_USERNAME}\" },"
    else
        VRCHAT_USERNAME_LINE=""
    fi

    if [ -v VRCHAT_PASSWORD ] ; then
        VRCHAT_PASSWORD_LINE="\"vrchat_password\": { \"secret\": \"${VRCHAT_PASSWORD}\" },"
    else
        VRCHAT_PASSWORD_LINE=""
    fi

    if [ -v VRCHAT_COOKIE ] ; then
        VRCHAT_COOKIE_LINE="\"vrchat_cookie\": { \"secret\": \"${VRCHAT_COOKIE}\" },"
    else
        VRCHAT_COOKIE_LINE=""
    fi

    echo "{
        \"enable_discord_status_message\": ${ENABLE_DISCORD_STATUS_MESSAGE},
        \"enable_vrchat\": ${ENABLE_VRCHAT},
        \"enable_discord\": ${ENABLE_DISCORD},
        \"discord_user_id\": { \"secret\": \"invalid\" },
        $SIMPLY_PLURAL_TOKEN_LINE
        $DISCORD_STATUS_MESSAGE_TOKEN_LINE
        $VRCHAT_USERNAME_LINE
        $VRCHAT_PASSWORD_LINE
        $VRCHAT_COOKIE_LINE
        \"system_name\": \"${SYSTEM_PUBLIC_NAME-null}\",
        \"wait_seconds\": ${SECONDS_BETWEEN_UPDATES-null}
    }"
}
export -f get_user_config_json


setup_test_user() {
    echo "Creating user ..."
    EMAIL="test@example.com"
    JSON="{
        \"email\": { \"inner\": \"$EMAIL\" },
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

    export JWT="$(echo "$JWT_JSON" | jq -r .inner)"
    export USER_ID="$(echo "$JWT" | cut -d'.' -f2 | base64 --decode | jq -r .sub)"
    echo "Received Jwt: $JWT"
    echo "User ID: $USER_ID"

    set_user_config_and_restart
    
    # echo "User config JSON: $JSON"

    echo "Getting user info ..."
    USER_INFO="$(
        curl -s --fail-with-body \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $JWT" \
            "$BASE_URL/api/user/info"
    )"
    [[ "$( echo "$USER_INFO" | jq -r .id.inner )" == "$USER_ID" ]]
    [[ "$( echo "$USER_INFO" | jq -r .email.inner )" == "$EMAIL" ]]

    echo "Test user setup complete."
}

set_user_config_and_restart() {
    echo "Setting config and restarting ..."
    JSON="$(get_user_config_json)"
    curl -s --fail-with-body \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $JWT" \
        -d "$JSON" \
        "$BASE_URL/api/user/config_and_restart"
}

get_updater_statuses() {
    curl -s --fail-with-body \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $JWT" \
        "$BASE_URL/api/updaters/status"
}
export -f get_updater_statuses
