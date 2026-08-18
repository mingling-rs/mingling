$skipDirs = @('.git', '.temp', 'target', 'node_modules', '.pnpm')
$selfPath = (Get-Item -LiteralPath $MyInvocation.MyCommand.Path).Directory.FullName

function Test-InSkipDir {
    param(
        [object]$Item
    )
    $path = if ($Item -is [string]) {
        $Item
    } elseif ($Item.PSPath) {
        $Item.PSPath -replace '^.*::', ''
    } else {
        $Item.FullName
    }

    $parts = $path.Split([System.IO.Path]::DirectorySeparatorChar)
    for ($i = 0; $i -lt $parts.Length - 1; $i++) {
        if ($parts[$i] -in $skipDirs) {
            return $true
        }
    }
    return $false
}

function Invoke-UnhideRecursive {
    param([string]$Path)
    Get-ChildItem -LiteralPath $Path -Force | ForEach-Object {
        if ($_.PSIsContainer) {
            if ($_.Name -in $skipDirs) {
                if ($_.Attributes -band [System.IO.FileAttributes]::Hidden) {
                    Write-Host "    -> unhiding skip directory (self only): `"$($_.FullName)`""
                    $_.Attributes = $_.Attributes -bxor [System.IO.FileAttributes]::Hidden
                }
                return
            }
            Invoke-UnhideRecursive $_.FullName
        } else {
            if ($_.Attributes -band [System.IO.FileAttributes]::Hidden) {
                Write-Host "    -> unhiding: `"$($_.FullName)`""
                $_.Attributes = $_.Attributes -bxor [System.IO.FileAttributes]::Hidden
            }
        }
    }
}

function Test-GitPathSkippable {
    param([string]$GitPath)
    $parts = $GitPath.Split(@('/', '\'))
    for ($i = 0; $i -lt $parts.Length - 1; $i++) {
        if ($parts[$i] -in $skipDirs) {
            return $true
        }
    }
    return $false
}

Write-Host "Step 1: Unhiding all files and directories (skipping $($skipDirs -join ', '))..."

Invoke-UnhideRecursive -Path (Get-Location).Path

Write-Host "Step 2: Hiding git-ignored items..."

git ls-files --others --ignored --exclude-standard | Where-Object {
    -not (Test-GitPathSkippable $_)
} | ForEach-Object {
    $itemPath = $_
    Write-Host "... checking: `"$itemPath`""
    $item = Get-Item $_ -Force -ErrorAction SilentlyContinue
    if (-not $item) { return }

    if ($item.FullName -eq $selfPath) { return }

    if (Test-InSkipDir $item) {
        Write-Host "    -> skipping (inside skip directory)"
        return
    }

    if ($item.PSIsContainer) {
        if (-not ($item.Attributes -band [System.IO.FileAttributes]::Hidden)) {
            Write-Host "    -> hiding directory (non-recursive)"
            $item.Attributes = $item.Attributes -bor [System.IO.FileAttributes]::Hidden
        }
    } else {
        if (-not ($item.Attributes -band [System.IO.FileAttributes]::Hidden)) {
            Write-Host "    -> hiding"
            $item.Attributes = $item.Attributes -bor [System.IO.FileAttributes]::Hidden
        }
    }
}

Write-Host "Step 3: Hiding dot-prefixed items..."
Get-ChildItem -Path . -Force -Directory | Where-Object { $_.Name -match '^\.' } | ForEach-Object {
    Write-Host "... checking: `"$($_.FullName)`""
    if (Test-InSkipDir $_) {
        Write-Host "    -> skipping (inside skip directory)"
        return
    }
    if (-not ($_.Attributes -band [System.IO.FileAttributes]::Hidden)) {
        Write-Host "    -> hiding directory"
        $_.Attributes = $_.Attributes -bor [System.IO.FileAttributes]::Hidden
    }
}

Get-ChildItem -Path . -Force -File | Where-Object { $_.Name -match '^\.' } | ForEach-Object {
    if ($_.FullName -eq $selfPath) { return }
    Write-Host "... checking: `"$($_.FullName)`""
    if (Test-InSkipDir $_) {
        Write-Host "    -> skipping (inside skip directory)"
        return
    }
    if (-not ($_.Attributes -band [System.IO.FileAttributes]::Hidden)) {
        Write-Host "    -> hiding file"
        $_.Attributes = $_.Attributes -bor [System.IO.FileAttributes]::Hidden
    }
}
