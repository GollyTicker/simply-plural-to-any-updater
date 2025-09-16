#!/bin/bash

set -euo pipefail

cargo update

(cd bridge-src-tauri && cargo update)

(cd frontend && npm install && ncu -u)
(cd bridge-frontend && npm install && ncu -u)
