#!/bin/bash

set -euo pipefail

sudo apt update

# MinGW toolchains for Windows targets
sudo apt-get install -y gcc-mingw-w64-x86-64 gcc-mingw-w64-i686

# patch discord-rich-presence
echo "Patching discord-rich-presence..."
rm -rf patched/discord-rich-presence
git clone https://github.com/vionya/discord-rich-presence patched/discord-rich-presence
( cd patched/discord-rich-presence && git -c advice.detachedHead=false checkout 0.2.5 )
patch patched/discord-rich-presence/src/activity.rs < patched/activity.rs.patch
patch patched/discord-rich-presence/src/discord_ipc.rs < patched/discord_ipc.rs.patch
echo "Patch OK!"

