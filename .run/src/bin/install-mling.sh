#!/bin/bash

cargo install --path mingling_cli

mkdir -p .temp/comp
cp .temp/target/release/*_comp.* .temp/comp/ 2>/dev/null || echo "No matching files found"
