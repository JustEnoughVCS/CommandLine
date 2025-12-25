# The JustEnoughVCS CommandLine Completion

Register-ArgumentCompleter -Native -CommandName jv -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $words = $commandAst.CommandElements | ForEach-Object { $_.ToString() }
    $currentIndex = $words.IndexOf($wordToComplete)
    if ($currentIndex -lt 0) { $currentIndex = $words.Count }

    $cmd = "jv"
    $subcmd = if ($words.Count -gt 1) { $words[1] } else { $null }
    $subsubcmd = if ($words.Count -gt 2) { $words[2] } else { $null }

    # Base commands
    $baseCommands = @(
        "create", "init", "direct", "unstain", "account", "update",
        "sheet", "status", "here", "move", "mv", "docs", "exit", "use", "sheets", "accounts",
        "as", "make", "drop", "track", "hold", "throw", "login",
        "jump", "align"
    )

    # Account subcommands
    $accountCommands = @("list", "as", "add", "remove", "movekey", "mvkey", "mvk", "genpub", "help")

    # Sheet subcommands
    $sheetCommands = @("list", "use", "exit", "make", "drop", "help", "align")

    # Completion for main command
    if ($currentIndex -eq 1) {
        return $baseCommands | Where-Object { $_ -like "$wordToComplete*" }
    }

    # Completion for account command
    if ($subcmd -eq "account" -or $subcmd -eq "acc") {
        if ($currentIndex -eq 2) {
            return $accountCommands | Where-Object { $_ -like "$wordToComplete*" }
        }

        switch ($subsubcmd) {
            { @("as", "remove", "mvkey", "mvk", "movekey", "genpub") -contains $_ } {
                if ($currentIndex -eq 3) {
                    $accounts = & $cmd account list --raw 2>$null
                    return $accounts | Where-Object { $_ -like "$wordToComplete*" }
                } elseif ($currentIndex -eq 4 -and (@("mvkey", "mvk", "movekey", "genpub") -contains $subsubcmd)) {
                    # File completion for key files
                    return Get-ChildItem -Name -File -Path "." | Where-Object { $_ -like "$wordToComplete*" }
                }
            }
            "add" {
                if ($currentIndex -eq 3) {
                    # No completion for account name
                    return @()
                } elseif ($currentIndex -eq 4 -and $wordToComplete.StartsWith("-")) {
                    # Complete --keygen option
                    if ("--keygen" -like "$wordToComplete*") {
                        return "--keygen"
                    }
                }
            }
            { @("-", "rm") -contains $_ } {
                if ($currentIndex -eq 3) {
                    $accounts = & $cmd account list --raw 2>$null
                    return $accounts | Where-Object { $_ -like "$wordToComplete*" }
                }
            }
        }
        return @()
    }

    # Completion for sheet command
    if ($subcmd -eq "sheet" -or $subcmd -eq "sh") {
        if ($currentIndex -eq 2) {
            return $sheetCommands | Where-Object { $_ -like "$wordToComplete*" }
        }

        switch ($subsubcmd) {
            { @("use", "drop") -contains $_ } {
                if ($currentIndex -eq 3) {
                    $sheets = & $cmd sheet list --raw 2>$null
                    return $sheets | Where-Object { $_ -like "$wordToComplete*" }
                }
            }
            "make" {
                if ($currentIndex -eq 3) {
                    $allSheets = & $cmd sheet list --all --raw 2>$null
                    return $allSheets | Where-Object { $_ -like "$wordToComplete*" }
                }
            }
            "align" {
                if ($currentIndex -eq 3) {
                    $alignItems = @("lost", "moved", "erased")
                    $unsolvedItems = & $cmd sheet align --unsolved --raw 2>$null
                    $completions = $alignItems + $unsolvedItems
                    return $completions | Where-Object { $_ -like "$wordToComplete*" }
                } elseif ($currentIndex -eq 4) {
                    $item = $words[3]
                    $alignOperations = @()
                    $createdItems = & $cmd sheet align --created --raw 2>$null

                    if ($item -eq "lost") {
                        $alignOperations = @("confirm")
                    } elseif ($item -like "lost:*") {
                        $alignOperations = @("confirm") + $createdItems
                    } elseif ($item -eq "moved" -or $item -like "moved:*") {
                        $alignOperations = @("local", "remote")
                    } elseif ($item -eq "erased" -or $item -like "erased:*") {
                        $alignOperations = @("confirm")
                    } else {
                        $alignOperations = @("local", "remote", "confirm") + $createdItems
                    }

                    return $alignOperations | Where-Object { $_ -like "$wordToComplete*" }
                }
            }
        }
        return @()
    }

    # Completion for align command
    if ($subcmd -eq "align") {
        if ($currentIndex -eq 2) {
            $alignItems = @("lost", "moved", "erased")
            $unsolvedItems = & $cmd sheet align --unsolved --raw 2>$null
            $completions = $alignItems + $unsolvedItems
            return $completions | Where-Object { $_ -like "$wordToComplete*" }
        } elseif ($currentIndex -eq 3) {
            $item = $words[2]
            $alignOperations = @()
            $createdItems = & $cmd sheet align --created --raw 2>$null

            if ($item -eq "lost") {
                $alignOperations = @("confirm")
            } elseif ($item -like "lost:*") {
                $alignOperations = @("confirm") + $createdItems
            } elseif ($item -eq "moved" -or $item -like "moved:*") {
                $alignOperations = @("local", "remote")
            } elseif ($item -eq "erased" -or $item -like "erased:*") {
                $alignOperations = @("confirm")
            } else {
                $alignOperations = @("local", "remote", "confirm") + $createdItems
            }

            return $alignOperations | Where-Object { $_ -like "$wordToComplete*" }
        }
        return @()
    }

    # Completion for login command
    if ($subcmd -eq "login") {
        if ($currentIndex -eq 2) {
            $accounts = & $cmd account list --raw 2>$null
            return $accounts | Where-Object { $_ -like "$wordToComplete*" }
        } elseif ($currentIndex -eq 3) {
            $ipHistory = & $cmd _ip_history 2>$null
            return $ipHistory | Where-Object { $_ -like "$wordToComplete*" }
        }
        return @()
    }

    # Completion for direct command
    if ($subcmd -eq "direct") {
        if ($currentIndex -eq 2) {
            $ipHistory = & $cmd _ip_history 2>$null
            return $ipHistory | Where-Object { $_ -like "$wordToComplete*" }
        }
        return @()
    }

    # Aliases completion
    switch ($subcmd) {
        "as" {
            if ($currentIndex -eq 2) {
                $accounts = & $cmd account list --raw 2>$null
                return $accounts | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        "use" {
            if ($currentIndex -eq 2) {
                $sheets = & $cmd sheet list --raw 2>$null
                return $sheets | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        "make" {
            if ($currentIndex -eq 2) {
                $allSheets = & $cmd sheet list --all --raw 2>$null
                return $allSheets | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        "drop" {
            if ($currentIndex -eq 2) {
                $sheets = & $cmd sheet list --raw 2>$null
                return $sheets | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        "docs" {
            if ($currentIndex -eq 2) {
                $docs = & $cmd docs list --raw 2>$null
                return $docs | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
        { @("move", "mv", "track", "hold", "throw") -contains $_ } {
            # File completion for file operations
            return Get-ChildItem -Name -File -Path "." | Where-Object { $_ -like "$wordToComplete*" }
        }
    }

    return @()
}
