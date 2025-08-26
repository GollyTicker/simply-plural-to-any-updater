#!/bin/bash

rm -rf frontend/node_modules || true
cargo clean || true
rm -rf patched/discord-rich-presence || true
