#!/bin/bash

set -euo pipefail

./release/cargo-build.sh

./target/release/sp2any --database-url todo
