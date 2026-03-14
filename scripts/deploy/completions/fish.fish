#!/usr/bin/env fish
function __jvn_fish_complete
    set -l cmdline (commandline -opc)
    set -l buffer (commandline -b)
    set -l cursor (commandline -C)

    set -l current_word ""
    set -l previous_word ""
    set -l word_index 0
    set -l char_count 0

    for i in (seq (count $cmdline))
        set word $cmdline[$i]
        set char_count (math $char_count + (string length "$word") + 1)

        if test $cursor -le $char_count
            set word_index $i
            set current_word $word
            if test $i -gt 1
                set previous_word $cmdline[(math $i - 1)]
            end
            break
        end
    end

    set -l args \
        -f "$buffer" \
        -C "$cursor" \
        -w "$current_word" \
        -p "$previous_word" \
        -c "$cmdline[1]" \
        -i "$word_index" \
        -a $cmdline

    set -l output (jvn_comp $args 2>/dev/null)
    if test "$output" = "_file_"
        __fish_complete_path "$current_word"
    else
        printf "%s\n" $output
    end
end

complete -c jvn -a '(__jvn_fish_complete)'
