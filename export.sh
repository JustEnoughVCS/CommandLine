#!/bin/bash

# Require : Cargo (Rust)

# Build
if cargo build --workspace --release; then
    # Export
    cargo run --manifest-path crates/build_helper/Cargo.toml --bin exporter
fi
