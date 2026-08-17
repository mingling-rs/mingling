# PowerShell completion script for <<<bin_name>>>
Register-ArgumentCompleter -Native -CommandName '<<<bin_name>>>' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $line = $commandAst.ToString()

    $elements = @()
    if ($commandAst.CommandElements.Count -gt 0) {
        $elements = $commandAst.CommandElements | ForEach-Object { $_.Value }
    }

    $commandName = if ($elements.Count -gt 0) { $elements[0] } else { "" }

    $currentWord = $wordToComplete
    $previousWord = ""
    $wordIndex = 0

    $found = $false
    for ($i = 0; $i -lt $elements.Count; $i++) {
        if ($elements[$i] -eq $currentWord) {
            $wordIndex = $i + 1
            if ($i -gt 0) {
                $previousWord = $elements[$i - 1]
            }
            $found = $true
            break
        }
    }

    if (-not $found) {
        $wordIndex = $elements.Count + 1
        if ($elements.Count -gt 0) {
            $previousWord = $elements[-1]
        }
    }

    $args = @(
        "-C", $cursorPosition.ToString()
        "-i", $wordIndex.ToString()
        "-F", "Powershell"
    )

    if ($line) {
        $args += "-f"
        $args += ($line -replace '-', '^')
    }
    if ($currentWord) {
        $args += "-w"
        $args += ($currentWord -replace '-', '^')
    }
    if ($previousWord) {
        $args += "-p"
        $args += ($previousWord -replace '-', '^')
    }
    if ($commandName) {
        $args += "-c"
        $args += ($commandName -replace '-', '^')
    }

    foreach ($element in $elements) {
        if ($element) {
            $args += "-a"
            $args += ($element -replace '-', '^')
        }
    }

    $originalEncoding = [Console]::OutputEncoding
    $originalPSEncoding = $OutputEncoding
    [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
    $OutputEncoding = [System.Text.Encoding]::UTF8

    $output = & <<<bin_name>>> __comp $args 2>&1

    [Console]::OutputEncoding = $originalEncoding
    $OutputEncoding = $originalPSEncoding

    $output = $output -replace "`r`n", "`n" -replace "`r", "`n"

    if (-not $output) {
        return @()
    }

    $lines = $output -split "`n"

    if ($lines.Count -eq 0) {
        return @()
    }

    $firstLine = $lines[0].Trim()

    if ($firstLine -eq "_file_") {
        if ($lines.Count -gt 1) {
            $fileSuggestions = $lines[1..($lines.Count-1)]
        } else {
            $fileSuggestions = @()
        }

        $completionResults = @()
        $fileSuggestions | ForEach-Object {
            $path = $_
            $isDirectory = $path.EndsWith([System.IO.Path]::DirectorySeparatorChar) -or $path.EndsWith('/')
            $completionType = if ($isDirectory) { 'ProviderContainer' } else { 'ProviderItem' }
            $completionResults += [System.Management.Automation.CompletionResult]::new($path, $path, $completionType, $path)
        }

        return $completionResults
    } else {
        $completionResults = @()

        foreach ($line in $lines) {
            $trimmedLine = $line.Trim()

            if ($trimmedLine -match '^([^$]+)\$\((.+)\)$') {
                $text = $matches[1]
                $description = $matches[2]
                $completionResults += [System.Management.Automation.CompletionResult]::new(
                    $text,
                    $text,
                    'ParameterValue',
                    $description
                )
            } else {
                $text = $trimmedLine
                $resultType = if ($text.StartsWith('-')) { 'ParameterName' } else { 'ParameterValue' }
                $completionResults += [System.Management.Automation.CompletionResult]::new(
                    $text,
                    $text,
                    $resultType,
                    $text
                )
            }
        }

        return $completionResults
    }
}
