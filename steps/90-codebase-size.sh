#!/bin/bash

set -euo pipefail

echo "===== CLOC ===="
cloc --exclude-dir=node_modules,target --exclude-ext=json,svg .

echo "===== Largest Rust files by LoC ==="
ls */*.rs */*/*.rs | grep -v "discord-rich-presence-patched" | xargs wc -l | sort -n -r | tail -n +2 | head -n 10
