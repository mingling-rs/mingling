$starting_dir = Get-Location
Get-ChildItem -Recurse -Filter "Cargo.toml" | ForEach-Object {
    $project_dir = $_.DirectoryName
    Push-Location $project_dir
    cargo test
    Pop-Location
}
Set-Location $starting_dir
