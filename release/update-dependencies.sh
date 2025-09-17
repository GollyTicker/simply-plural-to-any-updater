#!/bin/bash

set -euo pipefail

cargo update

(cd frontend && npm install --ignore-scripts)
