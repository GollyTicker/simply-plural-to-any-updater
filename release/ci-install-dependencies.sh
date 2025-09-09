#!/bin/bash

set -euo pipefail

sudo apt update

# Tests
sudo apt-get install -y oathtool

# MinGW toolchains for Windows targets
sudo apt-get install -y gcc-mingw-w64-x86-64 gcc-mingw-w64-i686

# Tauri
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

# Tauri e2e tests
cargo install tauri-driver --locked
sudo apt install -y webkit2gtk-driver

# patch discord-rich-presence
echo "Patching discord-rich-presence..."
rm -rf release/patched/discord-rich-presence
git clone https://github.com/vionya/discord-rich-presence release/patched/discord-rich-presence
( cd release/patched/discord-rich-presence && git -c advice.detachedHead=false checkout 1.0.0 )
patch release/patched/discord-rich-presence/src/activity.rs < release/patched/activity.rs.patch
patch release/patched/discord-rich-presence/src/discord_ipc.rs < release/patched/discord_ipc.rs.patch
echo "Patch OK!"


# todo. move these to a cacheable place for ci
# npm
(cd bridge-frontend && npm ci)
(cd frontend && npm ci)
