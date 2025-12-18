#!/bin/bash

set -euo pipefail

source test/source.sh

./steps/21-bridge-frontend-tauri-build.sh

echo "! Test assumes that sp2any-api is running !"

(cd bridge-frontend && npm run e2e)
