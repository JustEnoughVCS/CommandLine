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

# Use JV_OUTDATED_MINUTES to set the expiration time (in minutes), requires JV_AUTO_UPDATE to be enabled
# Next time the `jv` command is used, if the content is outdated, `jv update` will be automatically executed
# When the set number is < 0, timeout-based update is disabled
# When the set number = 0, update runs every time (not recommended)
# When the set number > 0, update according to the specified time
# If not set, the default is -1
# $env:JV_OUTDATED_MINUTES = "5"

# Use JV_TEXT_EDITOR to set text editor for `jv track --work` `jv align --work`
# DEFAULT: $EDITOR environment variable, falling back to "jvii" if not set
# $env:JV_TEXT_EDITOR = "nano"

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
