$starting_dir = Get-Location
python -m http.server 3000
Set-Location $starting_dir
