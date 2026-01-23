# Hide all dotfiles and git-ignored files before build
# Set working directory to parent of script's directory
Set-Location -Path (Join-Path $PSScriptRoot "..")

# First, unhide all files and directories in the current directory, but skip .temp and .git directories
Get-ChildItem -Path . -Force -Recurse -ErrorAction SilentlyContinue | Where-Object {
    $_.FullName -notmatch '\\.temp\\' -and $_.FullName -notmatch '\\.git\\'
} | ForEach-Object {
    attrib -h $_.FullName 2>&1 | Out-Null
}

# Get all dotfiles and directories
Get-ChildItem -Path . -Force -Recurse -ErrorAction SilentlyContinue | Where-Object {
    $_.Name -match '^\..*' -and $_.FullName -notmatch '\\\.\.$' -and $_.FullName -notmatch '\\\.$'
} | ForEach-Object {
    attrib +h $_.FullName 2>&1 | Out-Null
}

# Get git ignored files and hide them
if (Get-Command git -ErrorAction SilentlyContinue) {
    git status --ignored --short | ForEach-Object {
        if ($_ -match '^!!\s+(.+)$') {
            $ignoredPath = $matches[1]
            if (Test-Path $ignoredPath) {
                attrib +h $ignoredPath 2>&1 | Out-Null
            }
        }
    }
}
