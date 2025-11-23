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

###############
### ALIASES ###
###############

alias jvh='jv here'
alias jvu='jv update'
alias jvt='jv track'
alias jmv='jv move'

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
