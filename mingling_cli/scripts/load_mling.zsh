#!/bin/bash

MLING_SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

export PATH="$MLING_SCRIPT_DIR/bin:$PATH"

if [[ -f "$MLING_SCRIPT_DIR/scripts/mling_comp.zsh" ]]; then
    source "$MLING_SCRIPT_DIR/scripts/mling_comp.zsh"
fi

while IFS= read -r pkg_dir; do
    [[ -z "$pkg_dir" || ! -d "$pkg_dir" ]] && continue
    case ":$PATH:" in
        *":$pkg_dir:"*) ;;
        *) export PATH="$pkg_dir:$PATH" ;;
    esac
done < <(mingling-cli __loadpkgs_path 2>/dev/null)

while IFS= read -r script; do
    [[ "$script" == *.zsh && -f "$script" ]] || continue
    source "$script"
done < <(mingling-cli __loadpkgs_comp_scripts 2>/dev/null)
