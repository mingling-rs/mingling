#!/bin/bash

find . -name "Cargo.toml" -type f | while read -r cargo_file; do
    project_dir=$(dirname "$cargo_file")
    (cd "$project_dir" && cargo clippy --fix --allow-dirty --allow-no-vcs --quiet)
done
