cargo install --path mling

New-Item -ItemType Directory -Force -Path .temp/comp | Out-Null
# Copy all files containing _comp from the debug directory
Get-ChildItem .temp/target/release/*_comp* | ForEach-Object {
    Copy-Item $_.FullName .temp/comp/
}
