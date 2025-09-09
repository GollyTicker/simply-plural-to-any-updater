#!/bin/bash

set -euo pipefail

cargo clippy --allow-dirty --fix

