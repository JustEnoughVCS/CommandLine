#!/bin/bash

# Require : Cargo (Rust)

# Build
if FORCE_BUILD=$(date +%s) cargo build --workspace --release; then
    # Export
    if cargo run --manifest-path crates/build_helper/Cargo.toml --bin exporter; then
        # Copy compile_info.rs.template to compile_info.rs after successful export
        cp -f templates/compile_info.rs src/data/compile_info.rs
    fi
fi
