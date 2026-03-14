#!bin/bash
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Completion script
if [ -n "$BASH_VERSION" ]; then
    # Bash
    if [ -f "$SCRIPT_DIR/comp/jvn_bash.sh" ]; then
        source "$SCRIPT_DIR/comp/jvn_bash.sh"
    fi
elif [ -n "$ZSH_VERSION" ]; then
    # Zsh
    if [ -f "$SCRIPT_DIR/comp/jvn_zsh.sh" ]; then
        source "$SCRIPT_DIR/comp/jvn_zsh.sh"
    fi
elif [ -n "$FISH_VERSION" ]; then
    # Fish
    if [ -f "$SCRIPT_DIR/comp/jvn_fish.fish" ]; then
        source "$SCRIPT_DIR/comp/jvn_fish.fish"
    fi
fi

# Envirement
if [ -d "$SCRIPT_DIR/bin" ]; then
    export PATH="$SCRIPT_DIR/bin:$PATH"
fi
