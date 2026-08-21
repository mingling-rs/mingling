#!/bin/sh

branch=$(git rev-parse --abbrev-ref HEAD)
remote=$(git config --get "branch.$branch.remote")
hash=$(git rev-parse HEAD)

if [ -n "$remote" ]; then
    dir="dev/local/$remote/$branch"
else
    dir="dev/local/$branch"
fi
file="$dir/last"

mkdir -p "$dir"
echo "$hash" > "$file"
