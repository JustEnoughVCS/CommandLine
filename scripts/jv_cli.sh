#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

##############
### CONFIG ###
##############

# Use JV_LANG to set CLI language
# Supported: en, zh-CN
# export JV_LANG=en

# Use JV_AUTO_UPDATE to set auto content update (yes/no)
# After local operations that change Upstream Vault content
# Next `jv` command will auto-run `jv update`
export JV_AUTO_UPDATE=yes

# Use JV_TEXT_EDITOR to set text editor for `jv track --work` `jv align --work`
# DEFAULT: $EDITOR environment variable, falling back to "jvii" if not set
# export JV_TEXT_EDITOR=nano

###############
### ALIASES ###
###############

# Disable glob expansion for jv commands across shells
if [ -n "$BASH_VERSION" ]; then # Bash
    alias jv='set -f; command jv; set +f'
    alias jvt='set -f; command jv track; set +f'
    alias jmv='set -f; command jv move; set +f'
elif [ -n "$ZSH_VERSION" ]; then # Zsh
    alias jv='noglob jv'
    alias jvt='noglob jv track'
    alias jmv='noglob jv move'
elif [ -n "$FISH_VERSION" ]; then # Fish
    function jv {
        command jv $@
    }
    function jvt {
        command jv track $@
    }
    function jmv {
        command jv move $@
    }
fi

alias jvh='jv here'
alias jvu='jv update'

##################
### COMPLETION ###
##################

if [ -f "$SCRIPT_DIR/completions/bash/completion_jv.sh" ]; then
    source "$SCRIPT_DIR/completions/bash/completion_jv.sh"
fi
if [ -f "$SCRIPT_DIR/completions/bash/completion_jvv.sh" ]; then
    source "$SCRIPT_DIR/completions/bash/completion_jvv.sh"
fi

##################
### ENVIREMENT ###
##################

if [ -d "$SCRIPT_DIR/bin" ]; then
    export PATH="$SCRIPT_DIR/bin:$PATH"
fi
