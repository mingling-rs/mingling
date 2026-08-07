#!/bin/bash

set -e

cargo build --release --manifest-path mingling_cli/Cargo.toml

mkdir -p .temp/mling/bin .temp/mling/scripts

cp .temp/target/release/mling .temp/mling/bin/
cp .temp/target/release/mingling-cli .temp/mling/bin/

for comp in zsh sh fish; do
    cp ".temp/target/release/mingling-cli_comp.$comp" ".temp/mling/scripts/mling_comp.$comp"
done
cp mingling_cli/scripts/load_mling.zsh .temp/mling/
cp mingling_cli/scripts/load_mling.sh .temp/mling/
cp mingling_cli/scripts/load_mling.fish .temp/mling/
