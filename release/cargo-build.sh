#/bin/bash

set -euo pipefail

docker compose down || true

docker volume rm -f sp2any-data || true

docker compose pull

docker compose up sp2any-db -d

export DATABASE_URL="postgres://postgres:postgres@localhost:5432/sp2any"

await(){
    CONTAINER="$1"
    QUERY="$2"
    echo "Checking $CONTAINER for $QUERY ..."

    SECONDS=0
    set +e
    until docker logs "$CONTAINER" 2>&1 | grep -q "$QUERY"; do
        sleep 1
        echo " $SECONDS"

        if ((SECONDS > 20)); then
            echo "Aborting."
            exit 1
        fi
    done
    echo "Ok."
    set -e
}

await sp2any-db "listening on IPv4 address"

cargo build --release "$@"

# use cargo sqlx prepare now to update the hashes such that cargo build
# doesn't an active DB for future compilation
