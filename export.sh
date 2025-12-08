#!/bin/bash

# Build
cargo build --workspace --release

# Export
cargo run --manifest-path crates/build_helper/Cargo.toml --bin exporter
