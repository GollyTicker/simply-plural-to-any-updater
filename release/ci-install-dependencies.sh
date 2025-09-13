#!/bin/bash

set -euo pipefail

sudo apt update

# MinGW toolchains for Windows targets
sudo apt-get install -y gcc-mingw-w64-x86-64 gcc-mingw-w64-i686
