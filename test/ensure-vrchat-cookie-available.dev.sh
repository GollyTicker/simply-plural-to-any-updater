#!/bin/bash

set -euo pipefail

set +u
if [[  "$1" != "--automated" ]]; then # setup called when script is used manually on terminal to get a cookie
    export DISCORD_STATUS_MESSAGE_UPDATER_AVAILABLE=false
    ENABLE_DISCORD_STATUS_MESSAGE=false
    ENABLE_VRCHAT=true
    ENABLE_DISCORD=false

    source ./test/source.sh
fi
set -u

export BASE_URL="http://localhost:8080"

main() {
    stop_updater
    ./steps/10-backend-cargo-build.sh
    ./docker/local.start.sh > docker/logs/start.log 2>&1

    if check_vrc_cookie_works ; then
        return 0
    else
        echo "Needs new VRChat Cookie"
    fi

    setup_test_user

    attempt_login_without_cookie

    read_2fa_code_from_terminal

    provide_2fa_code_for_new_cookie

    echo "VRChat cookie retrieval done."
}


attempt_login_without_cookie() {
    echo "VRChat: Attempting login without cookie..."
    VRCHAT_CREDENTIALS="{\"username\":\"$VRCHAT_USERNAME\",\"password\":\"$VRCHAT_PASSWORD\"}"
    RESPONSE="$(
        curl -s --fail-with-body \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $JWT" \
            -d "$VRCHAT_CREDENTIALS" \
            "$BASE_URL/api/user/platform/vrchat/auth_2fa/request"
    )"
    echo "Response: $RESPONSE"
    METHOD="$( echo "$RESPONSE" | jq -r .Right.method )"
    TMP_COOKIE="$( echo "$RESPONSE" | jq -r .Right.tmp_cookie )"
    echo "Method: $METHOD"
    echo "tmp_cookie: $TMP_COOKIE"
}

read_2fa_code_from_terminal() {
    if [[ -v VRCHAT_TOTP ]]; then
        TFA_CODE="$(oathtool --totp --base32 "$VRCHAT_TOTP")"
        export TFA_CODE
        echo "Using generated 2FA code: $TFA_CODE"
        sleep 2s # sleep a bit to simulate user duration
    else
        echo "Enter the 2FA code from your auth-app / Email:"
        read -r TFA_CODE
        export TFA_CODE
    fi
}

provide_2fa_code_for_new_cookie() {
    echo "VRChat: Providing 2FA code for new cookie ..."
    REQUEST_BODY="{
        \"creds\": $VRCHAT_CREDENTIALS,
        \"code\":{\"inner\":\"$TFA_CODE\"},
        \"method\":\"$METHOD\",
        \"tmp_cookie\": \"$TMP_COOKIE\"
    }"
    NEW_COOKIE_JSON="$(
        curl -s --fail-with-body \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $JWT" \
            -d "$REQUEST_BODY" \
            "$BASE_URL/api/user/platform/vrchat/auth_2fa/resolve"
    )"
    echo "Response: $NEW_COOKIE_JSON"
    VRCHAT_COOKIE="$(echo "$NEW_COOKIE_JSON" | jq -r .cookie)"
    echo "Received new VRChat cookie:"
    echo "$VRCHAT_COOKIE"
    export VRCHAT_COOKIE
}

check_vrc_cookie_works() {
    echo "Checking that VRC Cookie works..."

    if [ ! -v VRCHAT_COOKIE ]; then
        echo "VRCHAT_COOKIE not defined."
        return 1
    fi

    RESPONSE="$(curl -s --fail-with-body "https://api.vrchat.cloud/api/1/auth/user" \
        --cookie "$VRCHAT_COOKIE" \
        -u "$VRCHAT_USERNAME:$VRCHAT_PASSWORD" \
        -H "User-Agent: SP2Any/0.1.0 does-not-exist-792374@gmail.com"
    )"

    STATUS="$( echo "$RESPONSE" | jq -r '.statusDescription' )"
    echo "Status: '$STATUS'"

    if [[ "$STATUS" != "null" ]]; then
        echo "Ok."
    else
        echo "Cookie doesn't work."
        false
    fi
}


stop_updater() {
    echo "stop_updater"
    ./docker/local.stop.sh > docker/logs/stop.log 2>&1
    echo "Stopped Updater."
}
trap stop_updater EXIT


# manual approach
works() {
    echo "======================= 1 ========================"
    curl -i 'https://vrchat.com/api/1/auth/user' \
        -H 'User-Agent: Mozilla/5.0 Firefox/142.0' \
        -H 'Accept: */*' \
        -u "$VRCHAT_USERNAME:$VRCHAT_PASSWORD"

    echo "Enter the 'auth=auth_cookie...' part:"
    read -r COOKIE
    export COOKIE
    read_2fa_code_from_terminal

    echo "======================= 2 ========================"
    curl -v 'https://vrchat.com/api/1/auth/twofactorauth/emailotp/verify' \
    -X POST \
    -H 'User-Agent: Mozilla/5.0 Firefox/142.0' \
    -H 'content-type: application/json' \
    -H "Cookie: $COOKIE" \
    --data-raw "{\"code\":\"$TFA_CODE\"}"
}


# works

main
