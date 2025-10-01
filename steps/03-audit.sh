#!/bin/bash

set -euo pipefail

set +e

echo "base-src audit"
(cd base-src && cargo audit)
EXIT_CODE="$?"

echo "src audit"
cargo audit
EXIT_CODE="$?"

echo "bridge-src-tauri audit"
(cd bridge-src-tauri && cargo audit)
EXIT_CODE="$?"

echo "frontend audit"
(cd frontend && npm audit)
# set EXIT_CODE as the max of te previous and $?
E="$?"
EXIT_CODE=$(( E > EXIT_CODE ? E : EXIT_CODE ))

echo "bridge-frontend audit"
(cd bridge-frontend && npm audit)
E="$?"
EXIT_CODE=$(( E > EXIT_CODE ? E : EXIT_CODE ))


echo "Exit Code: $EXIT_CODE"

exit "$EXIT_CODE"
