#!/usr/bin/env fish
function __<<<bin_name>>>_fish_complete
    set -l cmdline (commandline -opc)
    set -l buffer (commandline -b)
    set -l cursor (commandline -C)
    set -l current_token (commandline -ct)

    set -l current_word ""
    set -l previous_word ""
    set -l word_index 0
    set -l char_count 0

    set -l found false
    if test -n "$current_token"
        for i in (seq (count $cmdline))
            if test "$cmdline[$i]" = "$current_token"
                set word_index $i
                set current_word $current_token
                if test $i -gt 1
                    set previous_word $cmdline[(math $i - 1)]
                end
                set found true
                break
            end
        end
    end

    if not $found
        for i in (seq (count $cmdline))
            set word $cmdline[$i]
            if test $i -gt 1
                set char_count (math $char_count + 1)
            end
            set char_count (math $char_count + (string length -- "$word"))

            if test $cursor -le $char_count
                set word_index $i
                set current_word $word
                if test $i -gt 1
                    set previous_word $cmdline[(math $i - 1)]
                end
                break
            end
        end
    end

    if test $word_index -eq 0 -a (count $cmdline) -gt 0
        set word_index (count $cmdline)
        if test -n "$current_token" -a "$current_token" != "$cmdline[-1]"
            set current_word $current_token
        else
            set current_word ""
        end
        set previous_word $cmdline[-1]
    end

    if test $word_index -gt (count $cmdline)
        set word_index (count $cmdline)
    end

    set -l buffer_replaced (string replace -a "-" "^" -- "$buffer")
    set -l current_word_replaced (string replace -a "-" "^" -- "$current_word")
    set -l previous_word_replaced (string replace -a "-" "^" -- "$previous_word")

    set -l args
    set -a args -f "$buffer_replaced" -C "$cursor" -w "$current_word_replaced" -p "$previous_word_replaced"

    if test (count $cmdline) -gt 0
        set -a args -c "$cmdline[1]"
    else
        set -a args -c ""
    end

    set -a args -i "$word_index"

    if test (count $cmdline) -gt 0
        set -l all_words_replaced
        for word in $cmdline
            set -a all_words_replaced (string replace -a "-" "^" -- "$word")
        end

        if test -n "$current_token" -a "$current_word" = "$current_token"
            set -l found_in_cmdline false
            for word in $cmdline
                if test "$word" = "$current_token"
                    set found_in_cmdline true
                    break
                end
            end
            if not $found_in_cmdline -a $word_index -eq (math (count $cmdline) + 1)
                set -a all_words_replaced (string replace -a "-" "^" -- "$current_token")
            end
        end

        set -a args -a $all_words_replaced
    else
        set -a args -a ""
    end

    set -a args -F "fish"

    set -l output
    if not <<<bin_name>>> __comp $args 2>/dev/null | read -z output
        return
    end

    set -l trimmed_output (string trim -- "$output")
    if test "$trimmed_output" = "_file_"
        __fish_complete_path "$current_word"
        return 0
    else if test -n "$trimmed_output"
        string split -n \n -- "$output" | while read -l line
            test -n "$line" && echo "$line"
        end
        return 0
    end
    return 1
end

complete -c <<<bin_name>>> -a '(__<<<bin_name>>>_fish_complete)' -f
