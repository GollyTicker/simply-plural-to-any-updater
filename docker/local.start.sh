#!/bin/bash

set -euo pipefail

source docker/source.sh

export PATH_TO_CONFIG_JSON="${PATH_TO_CONFIG_JSON-./does-not-exist}"


./docker/local.stop.sh


COMPOSE="docker compose -f docker/local.compose.yml"

$COMPOSE build --pull

$COMPOSE up -d sp2any-db

await sp2any-db "listening on IPv4 address"

$COMPOSE up -d

await sp2any-webserver "Rocket has launched from"
