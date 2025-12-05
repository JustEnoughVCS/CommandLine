$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Definition

##############
### CONFIG ###
##############

# Use JV_LANG to set CLI language
# Supported: en, zh-CN
# $env:JV_LANG = "en"

# Use JV_AUTO_UPDATE to set auto content update (yes/no)
# After local operations that change Upstream Vault content
# Next `jv` command will auto-run `jv update`
$env:JV_AUTO_UPDATE = "yes"

###############
### ALIASES ###
###############

function jv {
    & (Get-Command jv -CommandType Application) @args
}

function jvh { jv here @args }
function jvu { jv update @args }
function jvt { jv track @args }
function jmv { jv move @args }

Set-Alias jvh jvh
Set-Alias jvu jvu
Set-Alias jvt jvt
Set-Alias jmv jmv

##################
### COMPLETION ###
##################

if (Test-Path "$SCRIPT_DIR\completions\powershell\completion_jv.ps1") {
    . "$SCRIPT_DIR\completions\powershell\completion_jv.ps1"
}
if (Test-Path "$SCRIPT_DIR\completions\powershell\completion_jvv.ps1") {
    . "$SCRIPT_DIR\completions\powershell\completion_jvv.ps1"
}

###################
### ENVIRONMENT ###
###################

if (Test-Path "$SCRIPT_DIR\bin") {
    $env:PATH = "$SCRIPT_DIR\bin;" + $env:PATH
}
