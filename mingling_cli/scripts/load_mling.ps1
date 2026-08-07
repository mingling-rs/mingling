$ScriptDir = if ($PSScriptRoot) { $PSScriptRoot } else { Split-Path -Parent $MyInvocation.MyCommand.Path }

$env:PATH = "$ScriptDir\bin;$env:PATH"

if (Test-Path -PathType Leaf "$ScriptDir\scripts\mling_comp.ps1") {
    . "$ScriptDir\scripts\mling_comp.ps1"
}

foreach ($pkgDir in (& mingling-cli __loadpkgs_path 2>$null)) {
    if (-not [string]::IsNullOrWhiteSpace($pkgDir) -and (Test-Path -PathType Container $pkgDir)) {
        if ($env:PATH -notmatch [regex]::Escape($pkgDir)) {
            $env:PATH = "$pkgDir;$env:PATH"
        }
    }
}

foreach ($script in (& mingling-cli __loadpkgs_comp_scripts 2>$null)) {
    if ($script -like '*.ps1' -and (Test-Path -PathType Leaf $script)) {
        . $script
    }
}
