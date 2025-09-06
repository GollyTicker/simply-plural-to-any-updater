#!/bin/bash

set -euo pipefail

export TAURI_APP_PATH="bridge-src-tauri"

force_vite() {
    sleep 1.5s
    echo "Forcing vite to render..."
    curl --no-progress-meter http://localhost:5173 >/dev/null
    echo "Ok."
}


force_vite &
cargo tauri dev

