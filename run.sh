#!/bin/bash

#
#         ██  ██████  ██    ██ ███    ██      ███████ ██   ██
#        ██   ██   ██ ██    ██ ████   ██      ██      ██   ██
#       ██    ██████  ██    ██ ██ ██  ██      ███████ ███████
#      ██     ██   ██ ██    ██ ██  ██ ██           ██ ██   ██
#  ██ ██      ██   ██  ██████  ██   ████  ██  ███████ ██   ██
#
#  You can go to [https://catilgrass.github.io/run] to install it
#                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

# Version: 0.1.2

cd "$(dirname "$0")" || exit 1

declare -A tools

for file in dev/run/src/bin/*.sh; do
    if [ -f "$file" ]; then
        name=$(basename "$file" .sh)
        tools["$name"]="sh"
    fi
done

for file in dev/run/src/bin/*; do
    if [ -f "$file" ]; then
        name=$(basename "$file")
        if [[ ! "$name" == *.* ]]; then
            tools["$name"]="binary"
        fi
    fi
done

for file in dev/run/src/bin/*.cs; do
    if [ -f "$file" ]; then
        name=$(basename "$file" .cs)
        tools["$name"]="cs"
    fi
done

for file in dev/run/src/bin/*.go; do
    if [ -f "$file" ]; then
        name=$(basename "$file" .go)
        tools["$name"]="go"
    fi
done

for file in dev/run/src/bin/*.nim; do
    if [ -f "$file" ]; then
        name=$(basename "$file" .nim)
        tools["$name"]="nim"
    fi
done

for file in dev/run/src/bin/*.pl; do
    if [ -f "$file" ]; then
        name=$(basename "$file" .pl)
        tools["$name"]="pl"
    fi
done

for file in dev/run/src/bin/*.py; do
    if [ -f "$file" ]; then
        name=$(basename "$file" .py)
        tools["$name"]="py"
    fi
done

for file in dev/run/src/bin/*.rb; do
    if [ -f "$file" ]; then
        name=$(basename "$file" .rb)
        tools["$name"]="rb"
    fi
done

for file in dev/run/src/bin/*.rs; do
    if [ -f "$file" ]; then
        name=$(basename "$file" .rs)
        tools["$name"]="rs"
    fi
done

for file in dev/run/src/bin/*.zig; do
    if [ -f "$file" ]; then
        name=$(basename "$file" .zig)
        tools["$name"]="zig"
    fi
done

get_sorted_names() {
    for name in "${!tools[@]}"; do
        first_char="${name:0:1}"
        if [[ "$first_char" =~ [A-Z] ]]; then
            case_pri="0"
        else
            case_pri="1"
        fi
        if [ "${tools[$name]}" = "sh" ]; then
            lang_pri="0"
        else
            lang_pri="1"
        fi
        echo "$case_pri$lang_pri $name"
    done | sort | while read -r _ n; do echo "$n"; done
}

show_list() {
    local highlight="$1"

    local total=${#sorted_names[@]}
    local num_w=${#total}

    local max_name=0
    for name in "${sorted_names[@]}"; do
        len=${#name}
        ((len > max_name)) && max_name=$len
    done

    local inner_w=$((2 + num_w + 1 + 1 + max_name + 2 + 6 + 1 + 2))
    ((inner_w < 38)) && inner_w=38

    local title="./run.sh <NUMBER/NAME> [ARGS...]"
    local title_len=${#title}
    local dash_total=$((inner_w - 2 - title_len))
    local dash_left=$((dash_total / 2))
    local dash_right=$((dash_total - dash_left))

    echo "┌$(printf '─%.0s' $(seq 1 $dash_left)) $title $(printf '─%.0s' $(seq 1 $dash_right))┐"
    printf "│%*s│\n" $inner_w ""

    local bold_blue=$'\033[1;34m'
    local reset=$'\033[0m'

    local lc_h=""
    if [ -n "$highlight" ]; then
        lc_h=$(echo "$highlight" | tr '[:upper:]' '[:lower:]')
    fi

    local i=1
    for idx in "${!sorted_names[@]}"; do
        local name="${sorted_names[$idx]}"
        local type="${tools[$name]}"
        local lang
        case "$type" in
            sh) lang="Shell";;
            binary) lang="Binary";;
            cs) lang="C#";;
            go) lang="Go";;
            nim) lang="Nim";;
            pl) lang="Perl";;
            py) lang="Python";;
            rb) lang="Ruby";;
            rs) lang="Rust";;
            zig) lang="Zig";;
        esac

        local display_name
        display_name=$(echo "$name" | tr '_-' '--')

        if [ -n "$highlight" ]; then
            local lc_dn
            lc_dn=$(echo "$display_name" | tr '[:upper:]' '[:lower:]')
            [[ "$lc_dn" == "$lc_h"* ]] || continue

            local hl_len=${#highlight}
            local prefix="${display_name:0:hl_len}"
            local rest="${display_name:hl_len}"
            display_name="${bold_blue}${prefix}${reset}${rest}"
        fi

        i=$((idx + 1))
        local num_part
        num_part=$(printf "  %-*d) " $num_w $i)
        local pad=$((max_name - ${#name}))
        local pad_str
        pad_str=$(printf '%*s' $pad '')
        local lang_part=" [$lang]"
        local visible_len=$(( ${#num_part} + ${#name} + pad + ${#lang_part} ))
        local outer_pad=$((inner_w - visible_len))
        printf "│%s%s%s%s%s│\n" "$num_part" "$display_name" "$pad_str" "$lang_part" "$(printf '%*s' $outer_pad '')"
    done

    printf "│%*s│\n" $inner_w ""
    echo "└$(printf '─%.0s' $(seq 1 $inner_w))┘"
}

if [ $# -eq 0 ]; then
    sorted_names=($(get_sorted_names))
    show_list ""
    exit 1
fi

target_name="$1"
shift

if [[ "$target_name" =~ ^[0-9]+$ ]]; then
    sorted=($(get_sorted_names))
    idx=$((target_name - 1))
    if [ "$idx" -ge 0 ] && [ "$idx" -lt "${#sorted[@]}" ]; then
        target_name="${sorted[$idx]}"
    else
        echo "Error: invalid number '$target_name', valid range is 1-${#sorted[@]}"
        exit 1
    fi
fi

if [ -z "${tools[$target_name]}" ]; then
    normalized_user=$(echo "$target_name" | tr '[:upper:]' '[:lower:]' | sed 's/[_. -]//g')
    found=""
    for existing_name in "${!tools[@]}"; do
        normalized_existing=$(echo "$existing_name" | tr '[:upper:]' '[:lower:]' | sed 's/[_. -]//g')
        if [ "$normalized_user" = "$normalized_existing" ]; then
            found="$existing_name"
            break
        fi
    done
    if [ -n "$found" ]; then
        target_name="$found"
    else
        sorted_names=($(get_sorted_names))
        lc_user=$(echo "$target_name" | tr '[:upper:]' '[:lower:]')
        hit=0
        for name in "${sorted_names[@]}"; do
            lc_name=$(echo "$name" | tr '_-' '--' | tr '[:upper:]' '[:lower:]')
            if [[ "$lc_name" == "$lc_user"* ]]; then
                hit=1
                break
            fi
        done
        if [ $hit -eq 1 ]; then
            show_list "$target_name"
            exit 1
        fi
        echo "Error: target '$target_name' does not exist"
        exit 1
    fi
fi

type="${tools[$target_name]}"

case "$type" in
    sh)
        chmod +x "dev/run/src/bin/$target_name.sh"
        "dev/run/src/bin/$target_name.sh" "$@"
        ;;
    binary)
        chmod +x "dev/run/src/bin/$target_name"
        "dev/run/src/bin/$target_name" "$@"
        ;;
    cs)
        temp_dir="dev/run/target/csproj/$target_name"
        mkdir -p "$temp_dir"
        cat > "$temp_dir/Directory.Build.props" <<'PROPS'
<Project>
  <PropertyGroup>
    <BaseOutputPath>$(MSBuildThisFileDirectory)../../csharp/bin</BaseOutputPath>
    <BaseIntermediateOutputPath>$(MSBuildThisFileDirectory)../../csharp/obj</BaseIntermediateOutputPath>
  </PropertyGroup>
</Project>
PROPS
        cat > "$temp_dir/$target_name.csproj" <<'CSPROJ'
<Project Sdk="Microsoft.NET.Sdk">

    <PropertyGroup>
        <OutputType>Exe</OutputType>
        <TargetFramework>net8.0</TargetFramework>
        <ImplicitUsings>enable</ImplicitUsings>
        <Nullable>enable</Nullable>
    </PropertyGroup>

</Project>
CSPROJ
        cp "dev/run/src/bin/$target_name.cs" "$temp_dir/Program.cs"
        dotnet run --project "$temp_dir/$target_name.csproj" -- "$@"
        ;;
    go)
        go run "dev/run/src/bin/$target_name.go" "$@"
        ;;
    nim)
        nim r --hints:off "dev/run/src/bin/$target_name.nim" "$@"
        ;;
    pl)
        perl "dev/run/src/bin/$target_name.pl" "$@"
        ;;
    py)
        python "dev/run/src/bin/$target_name.py" "$@"
        ;;
    rb)
        ruby "dev/run/src/bin/$target_name.rb" "$@"
        ;;
    rs)
        if [ ! -f "dev/run/Cargo.toml" ]; then
            cat > "dev/run/Cargo.toml" <<'EOF'
[package]
name = "run_rust"
version = "0.1.0"
edition = "2024"

[workspace]

[dependencies]
EOF
        fi
        cargo build --manifest-path "dev/run/Cargo.toml" --target-dir "dev/run/target" --bin "$target_name" --quiet
        "dev/run/target/debug/$target_name" "$@"
        ;;
    zig)
        zig run "dev/run/src/bin/$target_name.zig" "$@"
        ;;
esac
