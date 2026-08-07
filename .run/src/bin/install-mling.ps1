$ErrorActionPreference = "Stop"

cargo build --release --manifest-path mingling_cli/Cargo.toml

New-Item -ItemType Directory -Force -Path .temp/mling/bin, .temp/mling/scripts | Out-Null

Copy-Item .temp/target/release/mling.exe .temp/mling/bin/
Copy-Item .temp/target/release/mingling-cli.exe .temp/mling/bin/
Copy-Item .temp/target/release/mingling-cli_comp.ps1 .temp/mling/scripts/mling_comp.ps1
Copy-Item mingling_cli/scripts/load_mling.ps1 .temp/mling/
