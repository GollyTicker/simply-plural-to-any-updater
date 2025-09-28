#!/bin/bash

set -euo pipefail

export TAURI_APP_PATH="bridge-src-tauri"
export SP2ANY_BASE_URL="http://localhost:8080"

echo "SP2ANY_BASE_URL: $SP2ANY_BASE_URL"
echo "IF you want to send requsts against a backend,"
echo "ensure that the backend is running via source secrets + ./test/start-backend-for-bridge-frontend.sh"

echo ""
echo "ATTENTION! global config file is being used!"


force_vite() {
    sleep 1.5s
    echo "Forcing vite to render..."
    curl --no-progress-meter http://localhost:5173 >/dev/null
    echo "Ok."
}


force_vite &
cargo tauri dev

