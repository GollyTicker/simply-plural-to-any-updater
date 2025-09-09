#!/bin/bash

set -euo pipefail

set +e

echo "Rust cargo audit"
# RUSTSEC-2023-0071: No fix available. Also doesn't seem to really impact us currently.
cargo audit --ignore RUSTSEC-2023-0071
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
