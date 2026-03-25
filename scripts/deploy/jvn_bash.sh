#!/bin/bash
if [[ "${BASH_SOURCE[0]}" != "" ]]; then
    SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"
    SCRIPT_DIR="$(dirname "$SCRIPT_PATH")"
else
    # Fallback
    SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
fi

# Completion script
if [ -n "$BASH_VERSION" ]; then
    if [ -f "$SCRIPT_DIR/comp/jvn_bash.sh" ]; then
        source "$SCRIPT_DIR/comp/jvn_bash.sh"
    else
        echo "Error: Completion script not found at $SCRIPT_DIR/comp/jvn_bash.sh" >&2
    fi
fi

# Environment
if [ -d "$SCRIPT_DIR/bin" ]; then
    export PATH="$SCRIPT_DIR/bin:$PATH"
fi
