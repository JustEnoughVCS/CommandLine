# Require : Cargo (Rust), ISCC (Inno Setup)

# Set location to script directory
$scriptPath = $MyInvocation.MyCommand.Path
$scriptDir = Split-Path $scriptPath -Parent
Set-Location $scriptDir

# Check for ISCC
$isccPath = Get-Command ISCC -ErrorAction SilentlyContinue
if (-not $isccPath) {
    Write-Warning '"Inno Setup" not installed. (https://jrsoftware.org/isinfo.php)'
    exit
}

# Build
$env:FORCE_BUILD=$(Get-Date -Format 'mmss')
cargo build --workspace --release
if ($LASTEXITCODE -ne 0) {
    # Build failed
} else {
    # Build succeeded
    # Export
    if (cargo run --manifest-path tools/build_helper/Cargo.toml --bin exporter release) {
        Copy-Item -Path templates\compile_info.rs.template -Destination src\data\compile_info.rs -Force
        ISCC /Q .\setup\windows\setup_jv_cli.iss
    }
}
