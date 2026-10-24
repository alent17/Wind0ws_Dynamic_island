param([ValidateSet('Check', 'Connect', 'Restore')][string]$Action = 'Check')
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$players = @(Get-CimInstance Win32_Process -Filter "name='cloudmusic.exe'")
$candidates = @($players | ForEach-Object { $_.ExecutablePath })
$candidates += $env:ISLE_NETEASE_PATH
foreach ($key in @('HKCU:\Software\Microsoft\Windows\CurrentVersion\App Paths\cloudmusic.exe', 'HKLM:\Software\Microsoft\Windows\CurrentVersion\App Paths\cloudmusic.exe', 'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths\cloudmusic.exe')) {
    if (Test-Path -LiteralPath $key) { $candidates += (Get-Item -LiteralPath $key).GetValue('') }
}
$shell = New-Object -ComObject WScript.Shell
$desktop = [Environment]::GetFolderPath('Desktop')
$userFolders = @($desktop, [Environment]::GetFolderPath('StartMenu'), (Join-Path $env:APPDATA 'Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar'))
$repairLinks = @()
foreach ($folder in @($userFolders + @([Environment]::GetFolderPath('CommonStartMenu')))) {
    if ($folder -and (Test-Path -LiteralPath $folder)) {
        foreach ($link in Get-ChildItem -LiteralPath $folder -Filter '*.lnk' -Recurse -ErrorAction SilentlyContinue) {
            $target = $shell.CreateShortcut($link.FullName).TargetPath
            if ([IO.Path]::GetFileName($target) -ieq 'cloudmusic.exe') {
                $candidates += $target
                if ($userFolders -contains $folder) { $repairLinks += $link.FullName }
            }
        }
    }
}
foreach ($root in @(${env:ProgramFiles}, ${env:ProgramFiles(x86)}, $env:LOCALAPPDATA)) {
    if ($root) { $candidates += (Join-Path $root 'NetEase\CloudMusic\cloudmusic.exe') }
}
$exe = $candidates | Where-Object { $_ -and (Test-Path -LiteralPath $_ -PathType Leaf) -and [IO.Path]::GetFileName($_) -ieq 'cloudmusic.exe' } | Select-Object -First 1
if (!$exe) { throw 'NETEASE_NOT_FOUND' }
$shortcutPath = Join-Path $desktop 'NetEase Music (Isle).lnk'
if ($Action -ne 'Check') {
    # Persist launch arguments before interrupting playback.
    $shortcut = $shell.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = $exe
    $shortcut.Arguments = '--remote-debugging-port=9223 --remote-debugging-address=127.0.0.1'
    $shortcut.WorkingDirectory = Split-Path -LiteralPath $exe
    $shortcut.IconLocation = "$exe,0"
    $shortcut.Save()
    foreach ($linkPath in $repairLinks) {
        try {
            $existing = $shell.CreateShortcut($linkPath)
            if ($existing.TargetPath -ieq $exe) {
                $arguments = $existing.Arguments -replace '(^|\s)--remote-debugging-port(?:=|\s+)\d+(?=\s|$)', ' '
                $arguments = $arguments -replace '(^|\s)--remote-debugging-address(?:=|\s+)\S+', ' '
                $existing.Arguments = ($arguments.Trim() + ' --remote-debugging-port=9223 --remote-debugging-address=127.0.0.1').Trim()
                $existing.Save()
            }
        } catch { Write-Verbose "Could not update shortcut: $linkPath" }
    }
}
$hasConnectionArguments = @($players | Where-Object { $_.ExecutablePath -ieq $exe -and $_.CommandLine -match '(^|\s)--remote-debugging-port=9223(\s|$)' }).Count -gt 0
# Automatic restore never interrupts playback. A temporary CDP outage (or a
# player still starting) is not evidence that the process needs restarting.
$restartRequired = $Action -eq 'Connect' -and $players.Count -gt 0 -and !$hasConnectionArguments -and $env:ISLE_NETEASE_CONNECTED -ne '1'
if ($restartRequired) {
    foreach ($player in $players) {
        if ($player.ExecutablePath -ieq $exe) { Stop-Process -Id $player.ProcessId -Force -ErrorAction SilentlyContinue }
    }
    $remaining = @(Get-CimInstance Win32_Process -Filter "name='cloudmusic.exe'" | Where-Object { $_.ExecutablePath -ieq $exe })
    if ($remaining.Count) { throw 'NETEASE_RESTART_FAILED' }
}
if ($Action -eq 'Connect' -and ($players.Count -eq 0 -or $restartRequired)) {
    Start-Process -FilePath $exe -ArgumentList '--remote-debugging-port=9223', '--remote-debugging-address=127.0.0.1' -WorkingDirectory (Split-Path -LiteralPath $exe) -WindowStyle Hidden
}
$configured = $false
if (Test-Path -LiteralPath $shortcutPath) {
    $savedLink = $shell.CreateShortcut($shortcutPath)
    $configured = $savedLink.TargetPath -ieq $exe -and $savedLink.Arguments -match '(^|\s)--remote-debugging-port=9223(\s|$)'
}
@{ executablePath = $exe; shortcutPath = $shortcutPath; configured = $configured; running = ($players.Count -gt 0 -or $Action -eq 'Connect'); restarted = $restartRequired } | ConvertTo-Json -Compress
