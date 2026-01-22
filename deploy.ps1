# Require : Cargo (Rust), ISCC (Inno Setup)

# Set location to script directory
$scriptPath = $MyInvocation.MyCommand.Path
$scriptDir = Split-Path $scriptPath -Parent
Set-Location $scriptDir

# Test core library
cargo test --manifest-path ..\VersionControl\Cargo.toml --workspace
if ($LASTEXITCODE -ne 0) {
    Write-Warning "Core library tests failed. Aborting build."
    exit 1
}

# Test workspace
cargo test --workspace
if ($LASTEXITCODE -ne 0) {
    Write-Warning "Workspace tests failed. Aborting build."
    exit 1
}

# Check if git worktree is clean
$gitStatus = git status --porcelain
if ($gitStatus) {
    Write-Warning "Git worktree is not clean. Commit or stash changes before building."
    exit 1
}

# Check for ISCC
$isccPath = Get-Command ISCC -ErrorAction SilentlyContinue
if (-not $isccPath) {
    Write-Warning '"Inno Setup" not installed. (https://jrsoftware.org/isinfo.php)'
    exit 1
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
