#/bin/bash

set -euo pipefail

(cd frontend && npm ci --ignore-scripts)

cargo tauri build --verbose "$@"
