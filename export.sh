#!/bin/bash

# Require : Cargo (Rust)

# Build
if cargo build --workspace --release >/dev/null 2>&1; then
    # Export
    cargo run --manifest-path crates/build_helper/Cargo.toml --bin exporter
fi
