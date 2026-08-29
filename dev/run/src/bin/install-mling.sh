#!/bin/bash

set -e

cargo build --release --manifest-path mingling_cli/Cargo.toml

mkdir -p .temp/mling/bin .temp/mling/scripts

cp .temp/target/release/mling .temp/mling/bin/
cp .temp/target/release/mingling-cli .temp/mling/bin/

for comp in zsh sh fish elv nu ps1; do
    if [ -f ".temp/target/mingling/mling_comp.$comp" ]; then
        cp ".temp/target/mingling/mling_comp.$comp" ".temp/mling/scripts/mling_comp.$comp"
    fi
done

for script in load_mling.zsh load_mling.sh load_mling.fish load_mling.elv load_mling.nu load_mling.ps1; do
    if [ -f "mingling_cli/scripts/$script" ]; then
        cp "mingling_cli/scripts/$script" ".temp/mling/"
    fi
done
