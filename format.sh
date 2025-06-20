#!/bin/bash

set -euo pipefail

rustfmt --edition 2021 src/**.rs
