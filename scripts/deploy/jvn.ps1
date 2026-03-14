$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Definition

# Completion
$completionScript = Join-Path $SCRIPT_DIR "comp\jvn_pwsl.ps1"
if (Test-Path $completionScript) {
    . $completionScript
}

# Envirement
$binPath = Join-Path $SCRIPT_DIR "bin"
if (Test-Path $binPath) {
    $env:PATH = "$binPath;$env:PATH"
}
