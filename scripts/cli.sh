#!/bin/bash

# Get the real directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

##############
### CONFIG ###
##############

# Use JV_LANG to set CLI language
# Supported: en, zh-CN
# export JV_LANG=en

###############
### ALIASES ###
###############

alias jj='jv here'
alias jvu='jv update'
alias jvt='jv track'
alias jmv='jv move'

##################
### COMPLETION ###
##################

if [ -f "$SCRIPT_DIR/completion_jv.sh" ]; then
    source "$SCRIPT_DIR/completion_jv.sh"
fi
if [ -f "$SCRIPT_DIR/completion_jvv.sh" ]; then
    source "$SCRIPT_DIR/completion_jvv.sh"
fi

##################
### ENVIREMENT ###
##################

if [ -d "$SCRIPT_DIR/bin" ]; then
    export PATH="$SCRIPT_DIR/bin:$PATH"
fi
