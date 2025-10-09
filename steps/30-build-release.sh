#!/bin/bash

set -euo pipefail

TARGETS=(
    "x86_64-pc-windows-gnu"    # 64-bit Windows
    "x86_64-unknown-linux-gnu" # 64-bit Linux (glibc)
)

LINUX_TARGET="x86_64-unknown-linux-gnu"


cleanup_output() {
    OUT_DIR="target/release_builds"
    echo "Cleaning '$OUT_DIR'"
    rm -rf "$OUT_DIR" || true
    mkdir -p "${OUT_DIR}"
}


add_rust_targets() {
    for target in "${TARGETS[@]}"; do
        rustup target add "$target"
    done
}


build_binaries() {
    for target in "${TARGETS[@]}"; do
        echo "🛠️ sp2any-bridge $target"
        ./steps/22-bridge-frontend-tauri-release.sh --target "$target"
        BUILD_OUT_PATH="bridge-src-tauri/target/$target/release/bundle"
        if [[ "$target" == *"windows"* ]]; then
            cp -v "$BUILD_OUT_PATH"/*/*.exe "$OUT_DIR/SP2Any-Bridge-Windows-Setup.exe"
        else
            cp -v "$BUILD_OUT_PATH"/*/*.rpm "$OUT_DIR/SP2Any-Bridge.rpm"
            cp -v "$BUILD_OUT_PATH"/*/*.deb "$OUT_DIR/SP2Any-Bridge.deb"
        fi
        echo "✅ sp2any-bridge $target"

        echo ""
    done


    echo "🛠️ sp2any-global-manager $LINUX_TARGET"
    ./steps/12-backend-cargo-build.sh --release --bin sp2any-global-manager --target "$LINUX_TARGET"
    src_path="target/$LINUX_TARGET/release/sp2any-global-manager"
    dest_path="${OUT_DIR}/sp2any-global-manager"
    cp -v "$src_path" "$dest_path"
    echo "✅ sp2any-global-manager $target"

    echo ""

    echo "🛠️ sp2any-api $LINUX_TARGET"
    ./steps/12-backend-cargo-build.sh --release --target "$LINUX_TARGET"
    src_path="target/$LINUX_TARGET/release/sp2any"
    dest_path="${OUT_DIR}/sp2any-api"
    cp -v "$src_path" "$dest_path"
    echo "✅ sp2any-api $target"

    echo ""

    echo "🛠️ sp2any-frontend $LINUX_TARGET"
    ./steps/17-frontend-npm-build.sh
    tar -czvf "$OUT_DIR/sp2any-frontend.tar.gz" -C frontend/dist .
    echo "✅ sp2any-brontend $target"
}


main() {
    cleanup_output
    add_rust_targets
    build_binaries

    echo ""
    echo "✅✅✅ Build process finished. Output in: ${PWD}/${OUT_DIR}"
}


main
