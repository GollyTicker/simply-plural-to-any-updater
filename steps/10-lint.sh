#!/bin/bash

set -euo pipefail


rust_lint() {
    cargo clippy --allow-dirty --fix -- \
        -W clippy::pedantic \
        -W clippy::nursery \
        -W clippy::unwrap_used \
        -W clippy::expect_used \
        -A clippy::missing_errors_doc

    rustfmt --edition 2024 src/**.rs
}


web_frontend_lint() {
    # todo. add linting
    # npm run lint -- --fix
    npx prettier --write src
}



rust_lint

(cd bridge-src-tauri && rust_lint)

(cd frontend && web_frontend_lint)

(cd bridge-frontend && web_frontend_lint)
