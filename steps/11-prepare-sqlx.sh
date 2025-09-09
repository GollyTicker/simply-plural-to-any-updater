#!/bin/bash

set -euo pipefail

source docker/source.sh
export PATH_TO_CONFIG_JSON="./does-not-exist"

./docker/local.stop.sh || true

docker compose -f docker/local.compose.yml pull
docker compose -f docker/local.compose.yml up sp2any-db -d

await sp2any-db "listening on IPv4 address"

export DATABASE_URL="postgres://postgres:postgres@localhost:5432/sp2any"

rm -v .sqlx/*.json || true

./steps/12-backend-cargo-build.sh

cargo sqlx prepare

./docker/local.stop.sh

unset DATABASE_URL

# this build should use the prepared queries now
./steps/12-backend-cargo-build.sh

echo "Refreshed SQLx prepare."
