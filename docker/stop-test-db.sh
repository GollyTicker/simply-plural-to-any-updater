#!/bin/bash

set +e

docker container rm -f sp2any-db

docker compose -f docker/local.compose.yml down

docker volume rm -f sp2any-data

