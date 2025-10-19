#!/bin/bash

set -euo pipefail

(cd base-src && cargo test)
cargo test
