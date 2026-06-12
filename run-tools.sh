#!/bin/bash

cd "$(dirname "$0")" || exit 1

# Collect all available tool names
tools=()

if [ -d "dev_tools/scripts" ]; then
    for file in dev_tools/scripts/*.sh; do
        if [ -f "$file" ]; then
            tools+=("$(basename "$file" .sh)")
        fi
    done
    for file in dev_tools/scripts/*.py; do
        if [ -f "$file" ]; then
            tools+=("$(basename "$file" .py)")
        fi
    done
fi
if [ -d "dev_tools/src/bin" ]; then
    for file in dev_tools/src/bin/*.rs; do
        if [ -f "$file" ]; then
            tools+=("$(basename "$file" .rs)")
        fi
    done
fi

if [ $# -eq 0 ]; then
    echo "Available:"
    for i in "${!tools[@]}"; do
        printf "  [%2d]  %s\n" $((i + 1)) "${tools[$i]}"
    done
    exit 1
fi

target_bin="$1"
shift  # Remove the first argument (tool name), keep the rest as tool arguments

# Check if input is a number
if [[ "$target_bin" =~ ^[0-9]+$ ]]; then
    idx=$((target_bin - 1))
    if [ "$idx" -ge 0 ] && [ "$idx" -lt "${#tools[@]}" ]; then
        target_bin="${tools[$idx]}"
    else
        echo "Error: invalid number '$target_bin', valid range is 1-${#tools[@]}"
        exit 1
    fi
fi

target_script="dev_tools/scripts/${target_bin}.sh"
target_python="dev_tools/scripts/${target_bin}.py"
target_file="dev_tools/src/bin/${target_bin}.rs"

if [ -f "$target_script" ]; then
    chmod +x "$target_script"
    "$target_script" "$@"
elif [ -f "$target_python" ]; then
    python "$target_python" "$@"
elif [ -f "$target_file" ]; then
    cargo run --manifest-path dev_tools/Cargo.toml --bin "$target_bin" --quiet -- "$@"
else
    echo "Error: target '$target_bin' does not exist"
    exit 1
fi
