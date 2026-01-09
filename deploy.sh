#!/bin/bash

# Require : Cargo (Rust)

# Change to the directory where the script is located
cd "$(dirname "$0")" || exit 1

# Build
if FORCE_BUILD=$(date +%s) cargo build --workspace --release; then
    # Export
    if cargo run --manifest-path crates/build_helper/Cargo.toml --bin exporter; then
        # Copy compile_info.rs.template to compile_info.rs after successful export
        cp -f templates/compile_info.rs src/data/compile_info.rs
    fi
fi
