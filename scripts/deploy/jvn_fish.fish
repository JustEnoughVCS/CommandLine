#! /usr/bin/env fish
set SCRIPT_DIR (dirname (status --current-filename))

# Completion script
if test -f "$SCRIPT_DIR/comp/jvn_fish.fish"
    source "$SCRIPT_DIR/comp/jvn_fish.fish"
end

# Environment
if test -d "$SCRIPT_DIR/bin"
    set -gx PATH "$SCRIPT_DIR/bin" $PATH
end
