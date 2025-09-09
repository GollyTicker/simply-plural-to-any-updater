#!/bin/bash
#
# Script to build Rust executables for various targets, including Windows and Linux.
# Note: MinGW toolchains are required for Windows builds.

set -euo pipefail



TARGETS=(
    "x86_64-pc-windows-gnu"    # 64-bit Windows
    "x86_64-unknown-linux-gnu" # 64-bit Linux (glibc)
)

# extract project name from Cargo.toml
PROJECT_BINARY_NAME=$(grep -E '^name\s*=\s*".*"' Cargo.toml | head -n 1 | sed -E 's/name\s*=\s*"([^"]+)".*/\1/')

OUTPUT_DIR_BASE="target/release_builds"
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
    echo "Step 3: Building binaries for ${PROJECT_BINARY_NAME}..."

    for target in "${TARGETS[@]}"; do
        echo "[$target] "
        ./steps/12-backend-cargo-build.sh --release --target "$target"
        echo "[$target] Cargo build successful."

        # Determine PLATFORM and .exe suffix based on the target triple
        platform_label=""
        exe_suffix=""
        if [[ "$target" == *"-pc-windows-gnu"* ]]; then
            platform_label="Win"
            exe_suffix=".exe"
        elif [[ "$target" == *"-unknown-linux-gnu"* ]]; then
            platform_label="Linux"
        fi

        final_executable_name="SP2Any-${platform_label}${exe_suffix}"

        # Path to the binary produced by cargo
        src_path="target/${target}/release/${PROJECT_BINARY_NAME}${exe_suffix}"
        dest_path="${OUTPUT_DIR_BASE}/${final_executable_name}"

        cp "$src_path" "$dest_path"
        echo "[$target] Created binary at '${dest_path}'."
    done
}


main() {
    ensure_mingw_toolchains
    add_rust_targets
    build_binaries

    echo ""
    echo "Build process finished. Output in: ${PWD}/${OUTPUT_DIR_BASE}"
}

main
