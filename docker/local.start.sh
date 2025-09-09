#!/bin/bash

set -euo pipefail

source docker/source.sh


./docker/local.stop.sh

COMPOSE="docker compose -f docker/local.compose.yml"

$COMPOSE build --pull

$COMPOSE up -d sp2any-db

await sp2any-db "listening on IPv4 address"

$COMPOSE up -d

await sp2any-api "Rocket has launched from"

await sp2any-entrypoint "start worker processes"
