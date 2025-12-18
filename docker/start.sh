#!/bin/bash

set -euo pipefail

export PLURALSYNC_STAGE="$1"
source "docker/$PLURALSYNC_STAGE.env"

source docker/source.sh

./docker/stop.sh "$PLURALSYNC_STAGE"

COMPOSE="docker compose -f docker/docker.compose.yml"

$COMPOSE build --pull

$COMPOSE up -d pluralsync-db

await pluralsync-db "listening on IPv4 address"

$COMPOSE up -d

await pluralsync-api "Rocket has launched from"

await pluralsync-entrypoint "start worker processes"

await pluralsync-global-manager "Authenticated."
