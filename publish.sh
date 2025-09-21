#!/bin/bash
cd "$(dirname "$0")"

cargo build --workspace

if [ $? -eq 0 ]; then
    cargo run --package cli_publisher
else
    exit 1
fi
