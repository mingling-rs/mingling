#!/usr/bin/env bash
_<<<bin_name>>>_bash_completion() {
    local line="${COMP_LINE:0:COMP_POINT}"
    local cur="${line##* }"
    local prev=""
    local word_index=1

    local before="${line:0:$(( ${#line} - ${#cur} ))}"
    local -a before_words
    if [[ -n "$before" ]]; then
        read -ra before_words <<< "$before"
        word_index=$(( ${#before_words[@]} + 1 ))
        if [[ $word_index -gt 1 ]]; then
            prev="${before_words[${#before_words[@]}-1]}"
        fi
    fi

    local args=()
    args+=(-f "${COMP_LINE//-/^}")
    args+=(-C "$COMP_POINT")
    args+=(-w "${cur//-/^}")
    args+=(-p "${prev//-/^}")
    args+=(-c "${COMP_WORDS[0]//-/^}")
    args+=(-i "$word_index")
    args+=(-F "bash")

    for word in "${COMP_WORDS[@]}"; do
        args+=(-a "${word//-/^}")
    done

    local suggestions
    if suggestions=$(<<<bin_name>>> __comp "${args[@]}" 2>/dev/null); then
        if [ $? -eq 0 ]; then
            if [ "$suggestions" = "_file_" ]; then
                compopt -o default
                COMPREPLY=()
                return
            fi

            if [ -n "$suggestions" ]; then
                local -a all_suggestions filtered
                mapfile -t all_suggestions < <(printf '%s\n' "$suggestions")

                for suggestion in "${all_suggestions[@]}"; do
                    [ -z "$cur" ] || [[ "$suggestion" == "$cur"* ]] && filtered+=("$suggestion")
                done

                if [ ${#filtered[@]} -gt 0 ]; then
                    COMPREPLY=("${filtered[@]}")
                    if [[ "$cur" == *:* && "$COMP_WORDBREAKS" == *:* ]]; then
                        local colon_prefix="${cur%"${cur##*:}"}"
                        local -a ltrimmed=()
                        for suggestion in "${COMPREPLY[@]}"; do
                            ltrimmed+=("${suggestion#"$colon_prefix"}")
                        done
                        COMPREPLY=("${ltrimmed[@]}")
                    fi
                fi
                return
            fi
        fi
    fi

    COMPREPLY=()
}

complete -F _<<<bin_name>>>_bash_completion <<<bin_name>>>
