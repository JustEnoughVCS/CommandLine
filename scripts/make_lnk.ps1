$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

$deployPs1 = Join-Path $scriptDir "dev\deploy.ps1"
$devDeployPs1 = Join-Path $scriptDir "dev\dev_deploy.ps1"
$parentDir = Split-Path $scriptDir -Parent

if (Test-Path $deployPs1) {
    $linkPath = Join-Path $parentDir "deploy.lnk"
    if (Test-Path $linkPath) { Remove-Item $linkPath -Force }
    $WshShell = New-Object -ComObject WScript.Shell
    $shortcut = $WshShell.CreateShortcut($linkPath)
    $shortcut.TargetPath = $deployPs1
    $shortcut.Save()
}

if (Test-Path $devDeployPs1) {
    $linkPath = Join-Path $parentDir "dev.lnk"
    if (Test-Path $linkPath) { Remove-Item $linkPath -Force }
    $WshShell = New-Object -ComObject WScript.Shell
    $shortcut = $WshShell.CreateShortcut($linkPath)
    $shortcut.TargetPath = $devDeployPs1
    $shortcut.Save()
}
