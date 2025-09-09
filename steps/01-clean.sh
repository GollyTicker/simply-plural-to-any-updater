#!/bin/bash

rm -rf frontend/node_modules || true
rm -rf bridge-frontend/node_modules || true
(cd bridge-src-tauri && cargo clean) || true
cargo clean || true
rm -rf target/target/discord-rich-presence-patched || true
