#!/bin/bash

set -euo pipefail

echo "Build..."
./release/cargo-build.sh
echo "Done."

echo "Run:"

echo "TODO: set config values json!"
exit 1

./target/release/sps_status --no-gui

