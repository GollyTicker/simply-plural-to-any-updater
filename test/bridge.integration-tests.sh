#!/bin/bash

set -euo pipefail

source test/source.sh

echo "Checking that backend is running..."
await sp2any-api "Waiting ${SECONDS_BETWEEN_UPDATES}s for next update trigger..."

./steps/bridge-frontend-tauri-build.sh

(cd bridge-frontend && npm run e2e)
