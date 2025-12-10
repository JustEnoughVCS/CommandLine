# Require : Cargo (Rust), ISCC (Inno Setup)

# Build
cargo build --workspace --release
if ($LASTEXITCODE -ne 0) {
    # Build failed
} else {
    # Build succeeded
    # Export
    if (cargo run --manifest-path crates/build_helper/Cargo.toml --bin exporter) {
        ISCC /Q .\setup\windows\setup_jv_cli.iss
    }
}
