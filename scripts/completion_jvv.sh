#!/bin/bash

_jvv_completion() {
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
    local base_commands="create init here member service listen members -c -i -H -m -l -M"

    # Subcommands - Member
    local member_commands="register remove list help + - ls"

    # Subcommands - Service
    local service_commands="listen help"

    # Completion subcommands
    if [[ $cword -eq 1 ]]; then
        COMPREPLY=($(compgen -W "$base_commands" -- "$cur"))
        return 0
    fi

    # Completion member
    if [[ "$subcmd" == "member" || "$subcmd" == "-m" ]]; then
        if [[ $cword -eq 2 ]]; then
            COMPREPLY=($(compgen -W "$member_commands" -- "$cur"))
            return 0
        fi

        case "$subsubcmd" in
            "remove"|"-")
                if [[ $cword -eq 3 ]]; then
                    # Use jvv member list --raw
                    local members
                    members=$($cmd member list --raw 2>/dev/null)
                    COMPREPLY=($(compgen -W "$members" -- "$cur"))
                fi
                ;;
        esac
        return 0
    fi

    # Completion service
    if [[ "$subcmd" == "service" ]]; then
        if [[ $cword -eq 2 ]]; then
            COMPREPLY=($(compgen -W "$service_commands" -- "$cur"))
            return 0
        fi
        return 0
    fi

    # aliases
    case "$subcmd" in
        "-m")
            if [[ $cword -eq 2 ]]; then
                COMPREPLY=($(compgen -W "$member_commands" -- "$cur"))
            fi
            ;;
        "listen"|"-l")
            # listen command has no arguments to complete
            ;;
        "members"|"-M")
            # members command has no arguments to complete
            ;;
    esac
}

# Register completion function
complete -F _jvv_completion jvv
