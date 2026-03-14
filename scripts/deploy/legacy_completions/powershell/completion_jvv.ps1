# The JustEnoughVCS CommandLine Completion

Register-ArgumentCompleter -Native -CommandName jvv -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $words = $commandAst.CommandElements | ForEach-Object { $_.ToString() }
    $currentIndex = $words.IndexOf($wordToComplete)
    if ($currentIndex -lt 0) { $currentIndex = $words.Count }

    $cmd = "jvv"
    $subcmd = if ($words.Count -gt 1) { $words[1] } else { $null }
    $subsubcmd = if ($words.Count -gt 2) { $words[2] } else { $null }

    # Base commands
    $baseCommands = @("create", "init", "here", "member", "service", "listen", "members", "-c", "-i", "-H", "-m", "-l", "-M")

    # Member subcommands
    $memberCommands = @("register", "remove", "list", "help", "+", "-", "ls")

    # Service subcommands
    $serviceCommands = @("listen", "help")

    # Completion for main command
    if ($currentIndex -eq 1) {
        return $baseCommands | Where-Object { $_ -like "$wordToComplete*" }
    }

    # Completion for member command
    if ($subcmd -eq "member" -or $subcmd -eq "-m") {
        if ($currentIndex -eq 2) {
            return $memberCommands | Where-Object { $_ -like "$wordToComplete*" }
        }

        switch ($subsubcmd) {
            { @("remove", "-") -contains $_ } {
                if ($currentIndex -eq 3) {
                    $members = & $cmd member list --raw 2>$null
                    return $members | Where-Object { $_ -like "$wordToComplete*" }
                }
            }
        }
        return @()
    }

    # Completion for service command
    if ($subcmd -eq "service") {
        if ($currentIndex -eq 2) {
            return $serviceCommands | Where-Object { $_ -like "$wordToComplete*" }
        }
        return @()
    }

    # Aliases completion
    switch ($subcmd) {
        "-m" {
            if ($currentIndex -eq 2) {
                return $memberCommands | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        { @("listen", "-l", "members", "-M") -contains $_ } {
            # These commands have no arguments to complete
            return @()
        }
    }

    return @()
}
