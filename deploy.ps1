# Require : Cargo (Rust), ISCC (Inno Setup)

# Build
$env:CARGO_BUILD_RERUN_IF_ENV_CHANGED="FORCE_BUILD=$(Get-Date -Format 'mmss')"
cargo build --workspace --release
if ($LASTEXITCODE -ne 0) {
    # Build failed
} else {
    # Build succeeded
    # Export
    if (cargo run --manifest-path crates/build_helper/Cargo.toml --bin exporter) {
        Remove-Item -Path src\data\compile_info.rs -ErrorAction SilentlyContinue
        ISCC /Q .\setup\windows\setup_jv_cli.iss
    }
}
