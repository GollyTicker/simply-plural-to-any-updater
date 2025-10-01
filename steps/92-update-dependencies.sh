#!/bin/bash

set -euo pipefail

cargo update

(cd base-src && cargo update)
(cd bridge-src-tauri && cargo update)

(cd frontend && npm install --ignore-scripts && ncu -u)
(cd bridge-frontend && npm install --ignore-scripts && ncu -u)
