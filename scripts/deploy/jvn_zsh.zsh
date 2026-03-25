#!/bin/zsh
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Completion script
if [ -f "$SCRIPT_DIR/comp/jvn_zsh.zsh" ]; then
    source "$SCRIPT_DIR/comp/jvn_zsh.zsh"
fi

# Environment
if [ -d "$SCRIPT_DIR/bin" ]; then
    export PATH="$SCRIPT_DIR/bin:$PATH"
fi
