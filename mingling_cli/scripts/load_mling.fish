#!/usr/bin/env fish

set -g MLING_SCRIPT_DIR (path dirname (path resolve (status --current-filename)))

if not contains -- "$MLING_SCRIPT_DIR/bin" $PATH
    set -gx PATH "$MLING_SCRIPT_DIR/bin" $PATH
end

if test -f "$MLING_SCRIPT_DIR/scripts/mling_comp.fish"
    source "$MLING_SCRIPT_DIR/scripts/mling_comp.fish"
end

for pkg_dir in (mingling-cli __loadpkgs_path 2>/dev/null | string split '\n' | string match -rv '^$')
    if test -d "$pkg_dir"; and not contains -- "$pkg_dir" $PATH
        set -gx PATH "$pkg_dir" $PATH
    end
end

for script in (mingling-cli __loadpkgs_comp_scripts 2>/dev/null | string split '\n' | string match -rv '^$')
    if string match -q '*.fish' -- "$script"; and test -f "$script"
        source "$script"
    end
end
