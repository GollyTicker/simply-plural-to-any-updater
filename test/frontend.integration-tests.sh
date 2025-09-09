#!/bin/bash

set -euo pipefail

source test/source.sh

./steps/14-frontend-npm-build.sh

echo "Checking that backend is running..."
await sp2any-api "Waiting ${SECONDS_BETWEEN_UPDATES}s for next update trigger..."

(cd frontend && npm run e2e)
