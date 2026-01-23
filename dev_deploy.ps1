# Require : Cargo (Rust)

# Set location to script directory
$scriptPath = $MyInvocation.MyCommand.Path
$scriptDir = Split-Path $scriptPath -Parent
Set-Location $scriptDir

# Build
$env:FORCE_BUILD=$(Get-Date -Format 'mm')
cargo build --workspace
if ($LASTEXITCODE -ne 0) {
    # Build failed
} else {
    # Build succeeded
    # Export
    if (cargo run --manifest-path tools/build_helper/Cargo.toml --quiet --bin exporter debug) {
        Copy-Item -Path templates\compile_info.rs.template -Destination src\data\compile_info.rs -Force
    }
}
