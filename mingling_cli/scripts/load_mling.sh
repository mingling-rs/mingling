#!/bin/bash

MLING_SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

export PATH="$MLING_SCRIPT_DIR/bin:$PATH"

if [[ -f "$MLING_SCRIPT_DIR/scripts/mling_comp.sh" ]]; then
    source "$MLING_SCRIPT_DIR/scripts/mling_comp.sh"
fi

while IFS= read -r pkg_dir; do
    [[ -z "$pkg_dir" || ! -d "$pkg_dir" ]] && continue
    case ":$PATH:" in
        *":$pkg_dir:"*) ;;
        *) export PATH="$pkg_dir:$PATH" ;;
    esac
done < <(mingling-cli __loadpkgs_path 2>/dev/null)

while IFS= read -r script; do
    [[ "$script" == *.sh && -f "$script" ]] || continue
    source "$script"
done < <(mingling-cli __loadpkgs_comp_scripts 2>/dev/null)
