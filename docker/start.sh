#!/bin/bash

set -euo pipefail

export SP2ANY_STAGE="$1"
source "docker/$SP2ANY_STAGE.env"

source docker/source.sh

./docker/stop.sh "$SP2ANY_STAGE"

COMPOSE="docker compose -f docker/docker.compose.yml"

$COMPOSE build --pull

$COMPOSE up -d sp2any-db

await sp2any-db "listening on IPv4 address"

$COMPOSE up -d

await sp2any-api "Rocket has launched from"

await sp2any-entrypoint "start worker processes"

await sp2any-global-manager "Authenticated."
