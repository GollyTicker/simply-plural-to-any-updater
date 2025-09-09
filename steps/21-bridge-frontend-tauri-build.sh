#!/bin/bash

set -euo pipefail

export TAURI_APP_PATH="bridge-src-tauri"

cargo tauri build --debug --no-bundle
