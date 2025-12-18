#!/bin/bash

set -euo pipefail

source test/source.sh

./steps/17-frontend-npm-build.sh

echo "! Test assumes that pluralsync-api running and re-started on rebuild !"

export PLURALSYNC_BASE_URL=http://localhost:8080
(cd frontend && npm run e2e)
