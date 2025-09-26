#!/bin/bash
#
# Script to build Rust executables for various targets, including Windows and Linux.
# Note: MinGW toolchains are required for Windows builds.

set -euo pipefail



TARGETS=(
    "x86_64-pc-windows-gnu"    # 64-bit Windows
    "x86_64-unknown-linux-gnu" # 64-bit Linux (glibc)
)

OUTPUT_DIR_BASE="output"
echo "Cleaning '$OUTPUT_DIR_BASE'"
rm -rf "$OUTPUT_DIR_BASE" || true
mkdir -p "${OUTPUT_DIR_BASE}"



ensure_mingw_toolchains() {
    echo "Step 1: Ensure MinGW toolchains, which are needed for C Runtime libraries, are installed."
    dpkg -l gcc-mingw-w64-x86-64
}


add_rust_targets() {
    echo ""
    echo "Step 2: Add Rust targets using rustup."
    for target in "${TARGETS[@]}"; do
        rustup target add "$target"
    done
}


build_binaries() {
    echo ""
    echo "Step 3: Building binaries..."


    echo "== build bridge for linux and windows =="
    PROJECT_BINARY_NAME=sp2any-bridge
    for target in "${TARGETS[@]}"; do
        ./steps/22-bridge-frontend-tauri-release.sh --target "$target"
        dest_path="${OUTPUT_DIR_BASE}/sp2any-bridge/"
        mkdir -p "$dest_path"
        cp -v bridge-src-tauri/target/$target/release/bundle/*/*.{rpm,AppImage,deb,exe} "$dest_path" || true
    done


    echo "== build server for linux =="
    PROJECT_BINARY_NAME=sp2any
    ./steps/12-backend-cargo-build.sh --release --target "x86_64-unknown-linux-gnu"
    ./steps/17-frontend-npm-build.sh
    cp -vr ./frontend/dist "$OUTPUT_DIR_BASE/sp2any-frontend"
    src_path="target/x86_64-unknown-linux-gnu/release/sp2any"
    dest_path="${OUTPUT_DIR_BASE}/sp2any-api/sp2any"
    mkdir -p "$(dirname "$dest_path")"
    cp -v "$src_path" "$dest_path"
}


main() {
    ensure_mingw_toolchains
    add_rust_targets
    build_binaries

    echo ""
    echo "Build process finished. Output in: ${PWD}/${OUTPUT_DIR_BASE}"
}

main
