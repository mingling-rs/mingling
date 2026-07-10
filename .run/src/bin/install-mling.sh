#!/bin/bash

cargo install --path mling

mkdir -p .temp/comp
cp .temp/target/release/*_comp.* .temp/comp/ 2>/dev/null || echo "No matching files found"
