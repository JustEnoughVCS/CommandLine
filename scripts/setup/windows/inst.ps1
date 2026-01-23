# Execute uninstall script, attempt to remove leftover PROFILE entries
. ".\uninst.ps1"

# Calculate directories
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$parentDir = Split-Path -Parent $scriptDir

# Write configuration to PROFILE
Add-Content -Path $PROFILE -Value "# JustEnoughVCS - Begin #"
Add-Content -Path $PROFILE -Value ". `"$parentDir\jv_cli.ps1`""
Add-Content -Path $PROFILE -Value "# JustEnoughVCS - End #"

# Check dependencies, if OpenSSL is not found, show a prompt
try {
    $null = Get-Command openssl -ErrorAction Stop
} catch {
    $wshell = New-Object -ComObject Wscript.Shell
    $wshell.Popup("OpenSSL was not found on your computer. JustEnoughVCS key generation depends on OpenSSL.", 0, "JustEnoughVCS Installation", 0x0 + 0x30)
}
