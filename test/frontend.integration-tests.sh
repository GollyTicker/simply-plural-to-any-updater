#!/bin/bash

set -euo pipefail

source test/source.sh

./steps/17-frontend-npm-build.sh

echo "! Test assumes that sp2any-api is running !"

(cd frontend && npm run e2e)
