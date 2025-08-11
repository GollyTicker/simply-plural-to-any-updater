#!/bin/bash

set -euo pipefail

./release/cargo-build.sh

./target/release/sp2any --config dev/sp2any.json
