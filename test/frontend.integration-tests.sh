#!/bin/bash

set -euo pipefail

source test/source.sh

./steps/17-frontend-npm-build.sh

echo "! Test assumes that sp2any-api is running !"

# TODO. FIX ME. It actually doesn't use the frontend build outside of the docker! We want to use that!

(cd frontend && npm run e2e)
