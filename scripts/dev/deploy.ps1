# Require : Cargo (Rust), ISCC (Inno Setup)

# Set location to script directory
$scriptPath = $MyInvocation.MyCommand.Path
$scriptDir = Split-Path $scriptPath -Parent

# Run script to hide ignored files
$hideScriptPath = Join-Path $scriptDir "hide_ignored_file.ps1"
if (Test-Path $hideScriptPath) {
    & $hideScriptPath
} else {
    Write-Warning "hide_ignored_file.ps1 not found at $hideScriptPath"
}

Set-Location (Join-Path $scriptDir "..\..")

# Check for ISCC
$isccPath = Get-Command ISCC -ErrorAction SilentlyContinue
if (-not $isccPath) {
    Write-Warning '"Inno Setup" not installed. (https://jrsoftware.org/isinfo.php)'
    exit 1
}

# Check if core library exists
$coreLibPath = "..\VersionControl\"
if (-not (Test-Path $coreLibPath)) {
    Write-Warning "Core library not found at $coreLibPath. Aborting build."
    exit 1
}

# Test core library
Write-Host "Testing Core Library `".\..\VersionControl\Cargo.toml`""
cargo test --manifest-path ..\VersionControl\Cargo.toml --workspace --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Warning "Core library tests failed. Aborting build."
    exit 1
}

# Test workspace
Write-Host "Testing Command Line `".\Cargo.toml`""
cargo test --workspace --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Warning "Workspace tests failed. Aborting build."
    exit 1
}

# Check if main git worktree is clean
$gitStatus = git status --porcelain
if ($gitStatus) {
    Write-Warning "Git worktree is not clean. Commit or stash changes before building."
    exit 1
}

# Check if core library git worktree is clean
Push-Location $coreLibPath
$coreGitStatus = git status --porcelain
Pop-Location
if ($coreGitStatus) {
    Write-Warning "Core library git worktree is not clean. Commit or stash changes before building."
    exit 1
}

# Build
$env:FORCE_BUILD=$(Get-Date -Format 'mmss')
Write-Host "Building `".\Cargo.toml`""
cargo build --workspace --release --quiet
if ($LASTEXITCODE -ne 0) {
    # Build failed
} else {
    # Build succeeded
    # Export
    Write-Host "Deploying `".\.cargo\config.toml`""
    if (cargo run --manifest-path tools/build_helper/Cargo.toml --quiet --bin exporter release) {
        Copy-Item -Path templates\compile_info.rs.template -Destination src\data\compile_info.rs -Force
        Write-Host "Packing Installer `".\setup\windows\setup_jv_cli.iss`""
        ISCC /Q .\scripts\setup\windows\setup_jv_cli.iss
    }
}
