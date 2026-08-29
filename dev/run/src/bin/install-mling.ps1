$ErrorActionPreference = "Stop"

cargo build --release --manifest-path mingling_cli/Cargo.toml

New-Item -ItemType Directory -Force -Path .temp/mling/bin, .temp/mling/scripts | Out-Null

if (Test-Path .temp/target/release/mling.exe) { Copy-Item .temp/target/release/mling.exe .temp/mling/bin/ }
if (Test-Path .temp/target/release/mingling-cli.exe) { Copy-Item .temp/target/release/mingling-cli.exe .temp/mling/bin/ }

$compFiles = "ps1", "sh", "zsh", "fish", "elv", "nu"
foreach ($ext in $compFiles) {
    if (Test-Path ".temp/target/mingling/mling_comp.$ext") {
        Copy-Item ".temp/target/mingling/mling_comp.$ext" ".temp/mling/scripts/mling_comp.$ext"
    }
    if (Test-Path "mingling_cli/scripts/load_mling.$ext") {
        Copy-Item "mingling_cli/scripts/load_mling.$ext" ".temp/mling/"
    }
}
