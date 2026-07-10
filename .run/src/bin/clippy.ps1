$starting_dir = Get-Location
Get-ChildItem -Recurse -Filter "Cargo.toml" | ForEach-Object {
    $project_dir = $_.DirectoryName
    Push-Location $project_dir
    cargo clippy --quiet
    Pop-Location
}
Set-Location $starting_dir
