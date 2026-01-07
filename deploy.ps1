# Require : Cargo (Rust), ISCC (Inno Setup)

# Hide .cargo and .temp directories before build
if (Test-Path .cargo) {
    attrib +h .cargo
}
if (Test-Path .temp) {
    attrib +h .temp
}

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
    if (cargo run --manifest-path crates/build_helper/Cargo.toml --bin exporter) {
        Copy-Item -Path templates\compile_info.rs -Destination src\data\compile_info.rs -Force
        ISCC /Q .\setup\windows\setup_jv_cli.iss
    }
}
