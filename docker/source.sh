#!/bin/bash

set -euo pipefail

await() {
    CONTAINER="$1"
    QUERY="$2"
    echo "Checking '$CONTAINER' for '$QUERY' ..."

    SECONDS=0 # increments automatically
    set +e
    until (LOGS="$(docker logs "$CONTAINER" 2>&1 || true)"; echo "$LOGS" | grep "$QUERY" > /dev/null) ; do
        sleep 1
        echo -n "$SECONDS, "

        if ((SECONDS >= 20)); then
            echo "Aborting."
            exit 1
        fi
    done
    echo "Ok."
    set -e
}
export -f await

