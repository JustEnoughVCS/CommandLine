#!/bin/bash

# Require : Cargo (Rust)

# Build
export FORCE_BUILD=$(date +%Y-%m-%d)

if cargo build --workspace --release; then
    # Export
    if cargo run --manifest-path crates/build_helper/Cargo.toml --bin exporter; then
        # Delete compile_info.rs after successful export
        rm -f src/data/compile_info.rs
    fi
fi
