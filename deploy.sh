#!/bin/bash

# Require : Cargo (Rust)

# Change to the directory where the script is located
cd "$(dirname "$0")" || exit 1

# Test core library
cargo test --manifest-path ../VersionControl/Cargo.toml --workspace
if [ $? -ne 0 ]; then
    echo "Core library tests failed. Aborting build."
    exit 1
fi

# Test workspace
cargo test --workspace
if [ $? -ne 0 ]; then
    echo "Workspace tests failed. Aborting build."
    exit 1
fi

# Check if git worktree is clean
git_status=$(git status --porcelain)
if [ -n "$git_status" ]; then
    echo "Git worktree is not clean. Commit or stash changes before building."
    exit 1
fi

# Build
if FORCE_BUILD=$(date +%s) cargo build --workspace --release; then
    # Export
    if cargo run --manifest-path tools/build_helper/Cargo.toml --bin exporter release; then
        # Copy compile_info.rs.template to compile_info.rs after successful export
        cp -f templates/compile_info.rs src/data/compile_info.rs
    fi
fi
