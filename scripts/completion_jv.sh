#!/bin/bash
# The JustEnoughVCS CommandLine Completion

_jv_completion() {
    local cur prev words cword
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    words=("${COMP_WORDS[@]}")
    cword=$COMP_CWORD

    # Current
    local cmd="${words[0]}"
    local subcmd="${words[1]}"
    local subsubcmd="${words[2]}"

    # Subcommands
    local base_commands="create init direct unstain account update \
                         sheet status here import export in out \
                         move mv docs exit use sheets accounts \
                         as make drop track hold throw login \
                         jump align"

    # Subcommands - Account
    local account_commands="list as add remove movekey mvkey mvk genpub help"

    # Subcommands - Sheet
    local sheet_commands="list use exit make drop help align"

    # Subcommands - Sheet
    local sheet_commands="list use exit make drop help align"

    # Completion subcommands
    if [[ $cword -eq 1 ]]; then
        COMPREPLY=($(compgen -W "$base_commands" -- "$cur"))
        return 0
    fi

    # Completion account
    if [[ "$subcmd" == "account" || "$subcmd" == "acc" ]]; then
        if [[ $cword -eq 2 ]]; then
            COMPREPLY=($(compgen -W "$account_commands" -- "$cur"))
            return 0
        fi

        case "$subsubcmd" in
            "as"|"remove"|"mvkey"|"mvk"|"movekey"|"genpub")
                if [[ $cword -eq 3 ]]; then
                    # Use jv account list --raw
                    local accounts
                    accounts=$($cmd account list --raw 2>/dev/null)
                    COMPREPLY=($(compgen -W "$accounts" -- "$cur"))
                elif [[ $cword -eq 4 && ("$subsubcmd" == "mvkey" || "$subsubcmd" == "mvk" || "$subsubcmd" == "movekey" || "$subsubcmd" == "genpub") ]]; then
                    COMPREPLY=($(compgen -f -- "$cur"))
                fi
                ;;
            "add")
                if [[ $cword -eq 3 ]]; then
                    # No completion for account name, let user type it
                    COMPREPLY=()
                elif [[ $cword -eq 4 && "$cur" == -* ]]; then
                    # Complete --keygen option
                    COMPREPLY=($(compgen -W "--keygen" -- "$cur"))
                fi
                ;;
            "-"|"rm")
                if [[ $cword -eq 3 ]]; then
                    local accounts
                    accounts=$($cmd account list --raw 2>/dev/null)
                    COMPREPLY=($(compgen -W "$accounts" -- "$cur"))
                fi
                ;;
        esac
        return 0
    fi

    # Completion sheet
    if [[ "$subcmd" == "sheet" || "$subcmd" == "sh" ]]; then
        if [[ $cword -eq 2 ]]; then
            COMPREPLY=($(compgen -W "$sheet_commands" -- "$cur"))
            return 0
        fi

        case "$subsubcmd" in
            # Use jv sheet list --raw --all/--other
            "use"|"drop")
                if [[ $cword -eq 3 ]]; then
                    local sheets
                    sheets=$($cmd sheet list --raw 2>/dev/null)
                    COMPREPLY=($(compgen -W "$sheets" -- "$cur"))
                fi
                ;;
            "make")
                if [[ $cword -eq 3 ]]; then
                    local all_sheets
                    all_sheets=$($cmd sheet list --all --raw 2>/dev/null)
                    COMPREPLY=($(compgen -W "$all_sheets" -- "$cur"))
                fi
                ;;
            "align")
                if [[ $cword -eq 3 ]]; then
                    local align_items="lost moved"
                    local unsolved_items
                    unsolved_items=$($cmd sheet align --unsolved --raw 2>/dev/null)
                    COMPREPLY=($(compgen -W "$align_items $unsolved_items" -- "$cur"))
                elif [[ $cword -eq 4 ]]; then
                    local item="${words[3]}"
                    local align_operations=""
                    local created_items
                    created_items=$($cmd sheet align --created --raw 2>/dev/null)

                    if [[ "$item" == "lost" ]]; then
                        align_operations="confirm"
                    elif [[ "$item" == lost:* ]]; then
                        align_operations="confirm $created_items"
                    elif [[ "$item" == "moved" || "$item" == moved:* ]]; then
                        align_operations="local remote"
                    else
                        align_operations="local remote confirm $created_items"
                    fi

                    COMPREPLY=($(compgen -W "$align_operations" -- "$cur"))
                fi
                ;;
        esac
        return 0
    fi

    # Completion align
    if [[ "$subcmd" == "align" ]]; then
        if [[ $cword -eq 2 ]]; then
            local align_items="lost moved"
            local unsolved_items
            unsolved_items=$($cmd sheet align --unsolved --raw 2>/dev/null)
            COMPREPLY=($(compgen -W "$align_items $unsolved_items" -- "$cur"))
        elif [[ $cword -eq 3 ]]; then
            local item="${words[2]}"
            local align_operations=""
            local created_items
            created_items=$($cmd sheet align --created --raw 2>/dev/null)

            if [[ "$item" == "lost" ]]; then
                align_operations="confirm"
            elif [[ "$item" == lost:* ]]; then
                align_operations="confirm $created_items"
            elif [[ "$item" == "moved" || "$item" == moved:* ]]; then
                align_operations="local remote"
            else
                align_operations="local remote confirm $created_items"
            fi

            COMPREPLY=($(compgen -W "$align_operations" -- "$cur"))
        fi
        return 0
    fi

    # Completion login
    if [[ "$subcmd" == "login" ]]; then
        if [[ $cword -eq 2 ]]; then
            local accounts
            accounts=$($cmd account list --raw 2>/dev/null)
            COMPREPLY=($(compgen -W "$accounts" -- "$cur"))
        elif [[ $cword -eq 3 ]]; then
            local ip_history
            ip_history=$($cmd _ip_history 2>/dev/null)
            COMPREPLY=($(compgen -W "$ip_history" -- "$cur"))
        fi
        return 0
    fi

    # Completion direct
    if [[ "$subcmd" == "direct" ]]; then
        if [[ $cword -eq 2 ]]; then
            local ip_history
            ip_history=$($cmd _ip_history 2>/dev/null)
            COMPREPLY=($(compgen -W "$ip_history" -- "$cur"))
        fi
        return 0
    fi

    # aliases
    case "$subcmd" in
        "as")
            if [[ $cword -eq 2 ]]; then
                local accounts
                accounts=$($cmd account list --raw 2>/dev/null)
                COMPREPLY=($(compgen -W "$accounts" -- "$cur"))
            fi
            ;;
        "use")
            if [[ $cword -eq 2 ]]; then
                local sheets
                sheets=$($cmd sheet list --raw 2>/dev/null)
                COMPREPLY=($(compgen -W "$sheets" -- "$cur"))
            fi
            ;;
        "make")
            if [[ $cword -eq 2 ]]; then
                local all_sheets
                all_sheets=$($cmd sheet list --all --raw 2>/dev/null)
                COMPREPLY=($(compgen -W "$all_sheets" -- "$cur"))
            fi
            ;;
        "drop")
            if [[ $cword -eq 2 ]]; then
                local sheets
                sheets=$($cmd sheet list --raw 2>/dev/null)
                COMPREPLY=($(compgen -W "$sheets" -- "$cur"))
            fi
            ;;
        "docs")
            if [[ $cword -eq 2 ]]; then
                local docs
                docs=$($cmd docs list --raw 2>/dev/null)
                COMPREPLY=($(compgen -W "$docs" -- "$cur"))
            fi
            ;;
        "move"|"mv")
            COMPREPLY=($(compgen -f -- "$cur"))
            ;;
        "import"|"export"|"in"|"out"|"track"|"hold"|"throw")
            COMPREPLY=($(compgen -f -- "$cur"))
            ;;
    esac
}

complete -F _jv_completion jv
