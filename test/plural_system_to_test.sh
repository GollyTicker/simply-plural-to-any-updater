#!/bin/bash

export ANNALEA_ID="683f23e79aa188caf3000000"
export BORGNEN_ID="683f23f49aa189caf3000000"
export CLENNTRO_ID="683f24009aa18acaf3000000"
export DAENSSA_ID="683f24179aa18bcaf3000000"
export TEST_MEMBER_ID="683f243e9aa18ccaf3000000"


set_system_fronts_set() {
    SET="$1"

    clear_all_fronts

    if [[ "$SET" == "A" ]]; then
        set_to_front "$ANNALEA_ID"
        set_to_front "$BORGNEN_ID"
        set_to_front "$DAENSSA_ID"
    elif [[ "$SET" == "B" ]]; then
        set_to_front "$TEST_MEMBER_ID"
    else
        return 1
    fi
}


set_to_front() {
    MEMBER_ID="$1"
    FRONT_ID="$(openssl rand -hex 12)" # produces valid 24 hexdec digits
    curl --silent --fail-with-body -L "https://api.apparyllis.com/v1/frontHistory/$FRONT_ID" \
        -H 'Content-Type: application/json' \
        -H "Authorization: $SPS_API_WRITE_TOKEN" \
        -d "{
            \"customStatus\": \"\",
            \"custom\": false,
            \"live\": true,
            \"startTime\": 0,
            \"member\": \"$MEMBER_ID\"
        }" > /dev/null
    echo "Set member $MEMBER_ID to front (id: $FRONT_ID)."
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