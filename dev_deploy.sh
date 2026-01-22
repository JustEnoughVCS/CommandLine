#!/bin/bash

# Require : Cargo (Rust)

# Change to the directory where the script is located
cd "$(dirname "$0")" || exit 1

# Build
if FORCE_BUILD=$(date +%M) cargo build --workspace; then
    # Export
    if cargo run --manifest-path tools/build_helper/Cargo.toml --bin exporter debug; then
        # Copy compile_info.rs.template to compile_info.rs after successful export
        cp -f templates/compile_info.rs src/data/compile_info.rs
    fi
fi
