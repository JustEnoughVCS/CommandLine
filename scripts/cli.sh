#!/bin/bash

# Get the real directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Aliases
alias jvh='jv here'
alias jvu='jv update'
alias jvt='jv track'

# Completion
if [ -f "$SCRIPT_DIR/completion_jv.sh" ]; then
    source "$SCRIPT_DIR/completion_jv.sh"
fi
if [ -f "$SCRIPT_DIR/completion_jvv.sh" ]; then
    source "$SCRIPT_DIR/completion_jvv.sh"
fi

# Add bin directory to PATH
if [ -d "$SCRIPT_DIR/bin" ]; then
    export PATH="$SCRIPT_DIR/bin:$PATH"
fi
