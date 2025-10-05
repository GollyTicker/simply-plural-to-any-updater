#!/bin/bash

set -euo pipefail

source docker/source.sh
export SP2ANY_STAGE=local
export FRONTEND_DIST=./../frontend/dist
export PATH_TO_SP2ANY_API_EXEC=../target/debug/sp2any
export PATH_TO_SP2ANY_GLOBAL_MANAGER_EXEC=../target/debug/sp2any-global-manager

./docker/stop.sh local || true

docker compose -f docker/docker.compose.yml pull
docker compose -f docker/docker.compose.yml up sp2any-db -d

await sp2any-db "listening on IPv4 address"

export DATABASE_URL="postgres://postgres:postgres@localhost:5432/sp2any"

( cd docker && cargo sqlx migrate run )

rm -v .sqlx/*.json || true

./steps/12-backend-cargo-build.sh

cargo sqlx prepare

./docker/stop.sh local

unset DATABASE_URL

# this build should use the prepared queries now
./steps/12-backend-cargo-build.sh

echo "Refreshed SQLx prepare."
