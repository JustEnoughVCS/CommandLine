$profileContent = Get-Content $PROFILE -ErrorAction SilentlyContinue
if ($profileContent) {
    $startMarker = "# JustEnoughVCS - Begin #"
    $endMarker = "# JustEnoughVCS - End #"
    $newContent = @()
    $insideBlock = $false
    $foundStart = $false

    foreach ($line in $profileContent) {
        if ($line.Trim() -eq $startMarker) {
            $insideBlock = $true
            $foundStart = $true
            continue
        }
        if ($line.Trim() -eq $endMarker) {
            $insideBlock = $false
            continue
        }
        if (-not $insideBlock) {
            $newContent += $line
        }
    }

    if ($foundStart -and $insideBlock) {
        $newContent = @()
        $insideBlock = $false
        foreach ($line in $profileContent) {
            if ($line.Trim() -eq $startMarker) {
                $insideBlock = $true
                continue
            }
            if (-not $insideBlock) {
                $newContent += $line
            }
        }
    }

    $newContent | Set-Content $PROFILE
}
