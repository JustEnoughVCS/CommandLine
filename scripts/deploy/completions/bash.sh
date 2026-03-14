#!/usr/bin/env bash
_jvn_bash_completion() {
    local cur prev words cword

    local line="${COMP_LINE}"
    local point="${COMP_POINT}"

    if [ "${point}" -gt "${#line}" ]; then
        point="${#line}"
    fi

    words=($line)
    cword=0

    local i=0
    local pos=0
    for word in "${words[@]}"; do
        local word_start=$pos
        local word_end=$((pos + ${#word}))

        if [ "${point}" -ge "${word_start}" ] && [ "${point}" -le "${word_end}" ]; then
            cword=$i
            cur="${word}"
            break
        fi

        pos=$((pos + ${#word} + 1))
        i=$((i + 1))
    done

    if [ "${point}" -gt "${pos}" ]; then
        cword=${#words[@]}
        cur=""
    fi

    if [ "${cword}" -gt 0 ]; then
        prev="${words[$((cword-1))]}"
    else
        prev=""
    fi

    local args=(
        -f "$COMP_LINE"
        -C "$COMP_POINT"
        -w "$cur"
        -p "$prev"
        -c "${words[0]}"
        -i "$cword"
        -a "${words[@]}"
    )

    local suggestions
    if suggestions=$(jvn_comp "${args[@]}" 2>/dev/null); then
        if [ "$suggestions" = "_file_" ]; then
            compopt -o default
            COMPREPLY=()
        else
            mapfile -t COMPREPLY < <(printf '%s\n' "$suggestions")
        fi
    else
        COMPREPLY=()
    fi
}

complete -F _jvn_bash_completion jvn
