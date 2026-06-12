Set-Location -Path (Split-Path -Parent $MyInvocation.MyCommand.Path) -ErrorAction Stop

# Collect all available tool names
$tools = @()

if (Test-Path "dev_tools/scripts") {
    $scripts = Get-ChildItem -Path "dev_tools/scripts/*.ps1", "dev_tools/scripts/*.py"
    foreach ($script in $scripts) {
        if ($script -is [System.IO.FileInfo]) {
            $tools += $script.BaseName
        }
    }
}
if (Test-Path "dev_tools/src/bin") {
    $files = Get-ChildItem -Path "dev_tools/src/bin/*.rs"
    foreach ($file in $files) {
        if ($file -is [System.IO.FileInfo]) {
            $tools += $file.BaseName
        }
    }
}

if ($args.Count -eq 0) {
    Write-Host "Available:"
    for ($i = 0; $i -lt $tools.Count; $i++) {
        Write-Host ("  [{0,2}]  {1}" -f ($i + 1), $tools[$i])
    }
    exit 1
}

$target_name = $args[0]

# Check if input is a number
if ($target_name -match '^\d+$') {
    $idx = [int]$target_name - 1
    if ($idx -ge 0 -and $idx -lt $tools.Count) {
        $target_name = $tools[$idx]
    } else {
        Write-Host "Error: invalid number '$target_name', valid range is 1-$($tools.Count)"
        exit 1
    }
}

# Collect remaining arguments to pass to the script
$script_args = $args[1..$args.Count]

$script_file_ps1 = "dev_tools/scripts/${target_name}.ps1"
$script_file_py = "dev_tools/scripts/${target_name}.py"
$rust_file = "dev_tools/src/bin/${target_name}.rs"

if (Test-Path $script_file_ps1) {
    & $script_file_ps1 $script_args
} elseif (Test-Path $script_file_py) {
    python $script_file_py $script_args
} elseif (Test-Path $rust_file) {
    cargo run --manifest-path dev_tools/Cargo.toml --bin $target_name --quiet -- $script_args
} else {
    Write-Host "Error: target '$target_name' does not exist as a script or Rust program"
    exit 1
}
