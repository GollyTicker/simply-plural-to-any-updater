#!/bin/bash

set -euo pipefail

source docker/source.sh

./docker/stop-test-db.sh || true

docker compose -f docker/local.compose.yml pull
docker compose -f docker/local.compose.yml up sp2any-db -d

await sp2any-db "listening on IPv4 address"

export DATABASE_URL="postgres://postgres:postgres@localhost:5432/sp2any"

rm -v .sqlx/*.json || true

./release/cargo-build.sh

cargo sqlx prepare

./docker/stop-test-db.sh

unset DATABASE_URL

# this build should use the prepared queries now
./release/cargo-build.sh

echo "Refreshed SQLx prepare."
