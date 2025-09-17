#/bin/bash

set -euo pipefail

(cd frontend && npm ci --ignore-scripts)

# first -- moves from cargo args to tauri args
# second -- moves from tauri args to binary args
cargo tauri dev -- -- "$@"
