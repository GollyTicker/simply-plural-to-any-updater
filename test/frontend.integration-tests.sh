#!/bin/bash

set -euo pipefail

source test/source.sh

./steps/17-frontend-npm-build.sh

echo "! Test assumes that sp2any-api running and re-started on rebuild !"

export SP2ANY_BASE_URL=http://localhost:8080
(cd frontend && npm run e2e)
