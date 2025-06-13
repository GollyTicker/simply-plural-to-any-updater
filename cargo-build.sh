#/bin/bash

set -euo pipefail

export OUT_DIR="target"
cargo build --release "$@"