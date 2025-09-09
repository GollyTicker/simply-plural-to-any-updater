#!/bin/bash

set -euo pipefail

lint() {
    cargo clippy --allow-dirty --fix -- \
        -W clippy::pedantic \
        -W clippy::nursery \
        -W clippy::unwrap_used \
        -W clippy::expect_used \
        -A clippy::missing_errors_doc
}


rustfmt --edition 2024 src/**.rs bridge-src-tauri/**.rs

lint
(cd bridge-src-tauri && lint)

