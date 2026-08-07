#!/usr/bin/env zsh
_<<<bin_name>>>_completion() {
    local -a args
    local suggestions

    local buffer="$BUFFER"
    local cursor="$CURSOR"
    local current_word="${words[$CURRENT]}"
    local previous_word=""
    local command_name="${words[1]}"
    local word_index="$CURRENT"

    if [[ $CURRENT -gt 1 ]]; then
        previous_word="${words[$((CURRENT-1))]}"
    fi

    args=(
        -f "${buffer//-/^}"
        -C "$cursor"
        -w "${current_word//-/^}"
        -p "${previous_word//-/^}"
        -c "$command_name"
        -i "$word_index"
        -a "${(@)words//-/^}"
        -F "zsh"
    )

    suggestions=$(<<<bin_name>>> __comp "${args[@]}" 2>/dev/null)

    if [[ $? -eq 0 ]] && [[ -n "$suggestions" ]]; then
        local -a completions
        completions=(${(f)suggestions})

        if [[ "${completions[1]}" == "_file_" ]]; then
            shift completions
            _files
        else
            local -a parsed_completions
            for item in "${completions[@]}"; do
                if [[ "$item" =~ '^([^$]+)\$\((.+)\)$' ]]; then
                    parsed_completions+=("${match[1]//:/\\:}:${match[2]}")
                else
                    parsed_completions+=("${item//:/\\:}")
                fi
            done

            if (( $+functions[_describe] )); then
                _describe '<<<bin_name>>> commands' parsed_completions
            else
                local -a simple_completions
                for item in "${completions[@]}"; do
                    if [[ "$item" =~ '^([^$]+)\$\((.+)\)$' ]]; then
                        simple_completions+=("${match[1]}")
                    else
                        simple_completions+=("$item")
                    fi
                done
                compadd -a simple_completions
            fi
        fi
    fi
}

if (( $+functions[compdef] )); then
    compdef _<<<bin_name>>>_completion <<<bin_name>>>
    if [[ $? -ne 0 ]]; then
        compctl -K _<<<bin_name>>>_completion <<<bin_name>>>
    fi
else
    compctl -K _<<<bin_name>>>_completion <<<bin_name>>>
fi
