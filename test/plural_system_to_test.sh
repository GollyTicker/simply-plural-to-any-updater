#!/bin/bash

export ANNALEA_ID="683f23e79aa188caf3000000"
export BORGNEN_ID="683f23f49aa189caf3000000"
export CLENNTRO_ID="683f24009aa18acaf3000000"
export DAENSSA_ID="683f24179aa18bcaf3000000"
export TEST_MEMBER_ID="683f243e9aa18ccaf3000000"
export NOTIF_OK="68e1950560bb6cfa4a000000"
export NOTIF_NOT_OK="68e1952060bb6dfa4a000000"
export ARCHIVED_NOTIF_OK="68e195b960bb70fa4a000000"
export PBUCKET_MEMBER_NO="68e23ebed3877fbeb6000000"
export PBUCKET_MEMBER_YES="68e23eb0d3877ebeb6000000"
export CUSTOM_FRONT_1_ID="688d41c8aa2e477e53000000"


set_system_fronts_set() {
    SET="$1"

    clear_all_fronts

    if [[ "$SET" == "A" ]]; then
        set_to_front "$ANNALEA_ID"
        set_to_front "$BORGNEN_ID"
        set_to_front "$DAENSSA_ID"
        set_to_front "$CUSTOM_FRONT_1_ID"
    elif [[ "$SET" == "B" ]]; then
        set_to_front "$TEST_MEMBER_ID"
    elif [[ "$SET" == "C" ]]; then
        set_to_front "$NOTIF_OK"
        set_to_front "$NOTIF_NOT_OK"
        set_to_front "$ARCHIVED_NOTIF_OK"
    elif [[ "$SET" == "D" ]]; then
        set_to_front "$PBUCKET_MEMBER_NO"
        set_to_front "$PBUCKET_MEMBER_YES"
    else
        return 1
    fi
}


set_to_front() {
    FRONTER_ID="$1"
    FRONT_ID="$(openssl rand -hex 12)" # produces valid 24 hexdec digits
    UNIX_MILLIS_CURRENT="$(date +%s%3N)"
    UNIX_MILLIS_5_MIN_AGO="$((UNIX_MILLIS_CURRENT - 5*60*1000))"
    curl --silent --fail-with-body -L "https://api.apparyllis.com/v1/frontHistory/$FRONT_ID" \
        -H 'Content-Type: application/json' \
        -H "Authorization: $SPS_API_WRITE_TOKEN" \
        -d "{
            \"customStatus\": \"\",
            \"custom\": false,
            \"live\": true,
            \"startTime\": $UNIX_MILLIS_5_MIN_AGO,
            \"member\": \"$FRONTER_ID\"
        }" > /dev/null
    echo "Set member/custom-front $FRONTER_ID to front (id: $FRONT_ID)."
    rate_limiting_delay
}


clear_all_fronts() {
    echo "Clearing all active fronts."

    FRONTER_IDS="$(
        curl --silent \
            -L 'https://api.apparyllis.com/v1/fronters/' \
            -H "Authorization: $SPS_API_WRITE_TOKEN" |
            jq -r '.[].id'
    )"

    if [[ "$FRONTER_IDS" == "" ]]; then
        return 0
    fi

    while read fronter_id; do
        
        echo "Clearing front (id=$fronter_id)"
        
        curl --silent -L -X PATCH "https://api.apparyllis.com/v1/frontHistory/$fronter_id" \
            -H 'Content-Type: application/json' \
            -H "Authorization: $SPS_API_WRITE_TOKEN" \
            -d '{
                "live": false,
                "startTime": 0,
                "endTime": 15,
                "customStatus": "",
                "custom": false
            }'
        
        rate_limiting_delay

    done <<< "$FRONTER_IDS"
}

rate_limiting_delay() {
    sleep 0.5s
}