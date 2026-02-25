#!/bin/bash

# Require : Cargo (Rust)

# Start timing
start_time=$(date +%s.%N)

# Change to the directory where the script is located
cd "$(dirname "$0")/../../" || exit 1

# Check if core library exists
coreLibPath="../VersionControl/"
if [ ! -d "$coreLibPath" ]; then
    echo "Core library not found at $coreLibPath. Aborting build."
    exit 1
fi

# Test core library
echo "Testing Core Library \"../VersionControl/Cargo.toml\""
cargo test --manifest-path ../VersionControl/Cargo.toml --workspace --quiet > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "Core library tests failed. Aborting build."
    exit 1
fi

# Test workspace
echo "Testing Command Line \"./Cargo.toml\""
cargo test --workspace --quiet > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "Workspace tests failed. Aborting build."
    exit 1
fi

# Check if main git worktree is clean
git_status=$(git status --porcelain)
if [ -n "$git_status" ]; then
    echo "Git worktree is not clean. Commit or stash changes before building."
    exit 1
fi

# Check if core library git worktree is clean
pushd "$coreLibPath" > /dev/null
core_git_status=$(git status --porcelain)
popd > /dev/null
if [ -n "$core_git_status" ]; then
    echo "Core library git worktree is not clean. Commit or stash changes before building."
    exit 1
fi

# Build
echo "Building Command Line \"./Cargo.toml\""
if FORCE_BUILD=$(date +%s) cargo build --workspace --release --quiet > /dev/null 2>&1; then
    # Build succeeded
    # Export
    echo "Deploying Command Line \"./.cargo/config.toml\""
    if cargo run --manifest-path tools/build_helper/Cargo.toml --quiet --bin exporter release > /dev/null 2>&1; then
        # Copy compile_info.rs.template to compile_info.rs after successful export
        cp -f templates/compile_info.rs.template src/data/compile_info.rs
    fi
fi

# Calculate and display elapsed time
end_time=$(date +%s.%N)
elapsed_time=$(echo "$end_time - $start_time" | bc)
printf "Success (Finished in %.2fs)\n" $elapsed_time
