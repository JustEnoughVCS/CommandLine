. ".\uninst.ps1"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

$parentDir = Split-Path -Parent $scriptDir
Add-Content -Path $PROFILE -Value "`n# JustEnoughVCS - Begin #"
Add-Content -Path $PROFILE -Value ". `"$parentDir\jv_cli.ps1`""
Add-Content -Path $PROFILE -Value "# JustEnoughVCS - End #"
