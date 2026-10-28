param([ValidateSet('Check', 'Connect', 'Restore', 'OpenStandard', 'OpenEnhanced')][string]$Action = 'Check', [switch]$RepairCommonStartMenu)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$players = @(Get-CimInstance Win32_Process -Filter "name='cloudmusic.exe'")
$candidates = @($players | ForEach-Object { $_.ExecutablePath })
function Add-ConnectionArguments([string]$Arguments) {
    $Arguments = $Arguments -replace '(^|\s)--remote-debugging-port(?:=|\s+)\d+(?=\s|$)', ' '
    $Arguments = $Arguments -replace '(^|\s)--remote-debugging-address(?:=|\s+)\S+', ' '
    return ($Arguments.Trim() + ' --remote-debugging-port=9223 --remote-debugging-address=127.0.0.1').Trim()
}
function Repair-LaunchCommand([string]$Command) {
    # Only commands that directly launch a discovered NetEase executable.
    # Preserve file/URL placeholders and all unrelated startup arguments.
    foreach ($path in $executablePaths) {
        $pattern = '^\s*(?:"' + [regex]::Escape($path) + '"|' + [regex]::Escape($path) + '(?=\s|$))(?<args>.*)$'
        $match = [regex]::Match($Command, $pattern, [Text.RegularExpressions.RegexOptions]::IgnoreCase)
        if ($match.Success) { return '"' + $path + '" ' + (Add-ConnectionArguments $match.Groups['args'].Value) }
    }
    return $null
}
function Save-RegistryBackup([string]$Target, [string]$Name) {
    $backupKey = 'HKCU:\Software\Isle\NeteaseLaunchBackup'
    $backupName = $Target + '|' + $Name
    if (!(Test-Path -LiteralPath $backupKey)) { $null = New-Item -Path $backupKey }
    if ($null -eq (Get-Item -LiteralPath $backupKey).GetValue($backupName)) {
        $original = $null
        if (Test-Path -LiteralPath $Target) { $original = (Get-Item -LiteralPath $Target).GetValue($Name) }
        $record = @{ target = $Target; name = $Name; existed = ($null -ne $original); value = $original } | ConvertTo-Json -Compress
        $null = New-ItemProperty -LiteralPath $backupKey -Name $backupName -Value $record -PropertyType String -Force
    }
}
foreach ($key in @('HKCU:\Software\Microsoft\Windows\CurrentVersion\App Paths\cloudmusic.exe', 'HKLM:\Software\Microsoft\Windows\CurrentVersion\App Paths\cloudmusic.exe', 'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths\cloudmusic.exe')) {
    if (Test-Path -LiteralPath $key) { $candidates += (Get-Item -LiteralPath $key).GetValue('') }
}
$shell = New-Object -ComObject WScript.Shell
$desktop = [Environment]::GetFolderPath('Desktop')
$userFolders = @($desktop, [Environment]::GetFolderPath('StartMenu'), [Environment]::GetFolderPath('Startup'), (Join-Path $env:APPDATA 'Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar'))
$repairLinks = @()
$personalCopies = @{}
$protectedStartMenu = @()
$commonStartMenu = [Environment]::GetFolderPath('CommonStartMenu')
$commonToPersonal = @{}
foreach ($pair in @(@('CommonDesktopDirectory','Desktop'), @('CommonStartMenu','StartMenu'), @('CommonStartup','Startup'))) {
    $common = [Environment]::GetFolderPath($pair[0])
    $personal = [Environment]::GetFolderPath($pair[1])
    if ($common -and $personal) { $commonToPersonal[$common] = $personal }
}
foreach ($folder in @($userFolders + @([Environment]::GetFolderPath('CommonStartMenu'), [Environment]::GetFolderPath('CommonDesktopDirectory'), [Environment]::GetFolderPath('CommonStartup'))) | Select-Object -Unique) {
    if ($folder -and (Test-Path -LiteralPath $folder)) {
        foreach ($link in Get-ChildItem -LiteralPath $folder -Filter '*.lnk' -Recurse -ErrorAction SilentlyContinue) {
            # A broken/unreadable shortcut belonging to any app must not abort
            # NetEase discovery for the whole desktop or Start menu.
            try { $target = $shell.CreateShortcut($link.FullName).TargetPath } catch { continue }
            if ([IO.Path]::GetFileName($target) -ieq 'cloudmusic.exe') {
                $candidates += $target
                $repairLinks += $link.FullName
                if ($commonToPersonal.ContainsKey($folder)) {
                    $relative = $link.FullName.Substring($folder.Length).TrimStart('\')
                    $personalCopies[$link.FullName] = Join-Path $commonToPersonal[$folder] $relative
                }
            }
        }
    }
}
# Prefer the current registration and ordinary shortcuts over a saved path that
# may still point to a leftover installation after a client update.
$candidates += $env:ISLE_NETEASE_PATH
foreach ($root in @(${env:ProgramFiles}, ${env:ProgramFiles(x86)}, $env:LOCALAPPDATA)) {
    if ($root) { $candidates += (Join-Path $root 'NetEase\CloudMusic\cloudmusic.exe') }
}
$executablePaths = @($candidates | Where-Object { $_ -and (Test-Path -LiteralPath $_ -PathType Leaf) -and [IO.Path]::GetFileName($_) -ieq 'cloudmusic.exe' } | Select-Object -Unique)
$exe = $executablePaths | Select-Object -First 1
if (!$exe) { throw 'NETEASE_NOT_FOUND' }
if ($Action -eq 'OpenStandard') {
    # Discovery above is read-only. Launch the executable directly so a legacy
    # protocol handler or shortcut cannot silently add CDP arguments.
    Start-Process -FilePath $exe -WorkingDirectory (Split-Path -LiteralPath $exe) -WindowStyle Hidden
    @{ executablePath = $exe; shortcutPath = ''; configured = $false; launchEntries = 0; running = $true; restarted = $false } | ConvertTo-Json -Compress
    return
}
$launchEntries = 0
# Repair only existing NetEase Run values; do not enable autostart for users
# who disabled it. File/protocol overrides are scoped to this Windows user.
$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
if (Test-Path -LiteralPath $runKey) {
    $run = Get-Item -LiteralPath $runKey
    foreach ($name in $run.GetValueNames()) {
        $before = $run.GetValue($name)
        if ($before -isnot [string]) { continue }
        $after = Repair-LaunchCommand $before
        if (!$after) { continue }
        if ($before -ceq $after) { $launchEntries++; continue }
        if ($Action -ne 'Check') {
            try {
                Save-RegistryBackup $runKey $name
                $null = New-ItemProperty -LiteralPath $runKey -Name $name -Value $after -PropertyType String -Force
                $launchEntries++
            } catch { Write-Verbose "Could not update NetEase autostart: $name" }
        }
    }
}
$classes = @('orpheus', 'cloudmusic', 'Applications\cloudmusic.exe')
$classes += @('aac','ape','cda','cue','flac','m4a','mp3','ncm','ogg','wav','wma') | ForEach-Object { 'cloudmusic.' + $_ }
foreach ($class in $classes) {
    $effective = 'Registry::HKEY_CLASSES_ROOT\' + $class + '\shell\open\command'
    if (!(Test-Path -LiteralPath $effective)) { continue }
    $before = (Get-Item -LiteralPath $effective).GetValue('')
    $after = Repair-LaunchCommand $before
    if (!$after) { continue }
    if ($before -ceq $after) { $launchEntries++; continue }
    if ($Action -ne 'Check') {
        $target = 'HKCU:\Software\Classes\' + $class + '\shell\open\command'
        try {
            Save-RegistryBackup $target ''
            if (!(Test-Path -LiteralPath $target)) { $null = New-Item -Path $target }
            Set-Item -LiteralPath $target -Value $after
            $launchEntries++
        } catch { Write-Verbose "Could not update NetEase launch association: $class" }
    }
}
$shortcutPath = Join-Path $desktop 'NetEase Music (Isle).lnk'
if ($Action -ne 'Check') {
    # Keep an existing legacy shortcut usable; only create a fallback when no
    # normal launch entry is available. Ordinary launchers remain the default.
    if ((Test-Path -LiteralPath $shortcutPath) -or ($repairLinks.Count -eq 0 -and $launchEntries -eq 0)) {
        $shortcut = $shell.CreateShortcut($shortcutPath)
        $arguments = Add-ConnectionArguments $shortcut.Arguments
        $directory = Split-Path -LiteralPath $exe
        if ($shortcut.TargetPath -ine $exe -or $shortcut.Arguments -cne $arguments -or $shortcut.WorkingDirectory -ine $directory -or $shortcut.IconLocation -ine "$exe,0") {
            $shortcut.TargetPath = $exe
            $shortcut.Arguments = $arguments
            $shortcut.WorkingDirectory = $directory
            $shortcut.IconLocation = "$exe,0"
            $shortcut.Save()
        }
    }
    foreach ($linkPath in ($repairLinks | Select-Object -Unique)) {
        try {
            $existing = $shell.CreateShortcut($linkPath)
            if ($executablePaths -contains $existing.TargetPath) {
                $arguments = Add-ConnectionArguments $existing.Arguments
                if ($existing.Arguments -cne $arguments) { $existing.Arguments = $arguments; $existing.Save() }
                $launchEntries++
            }
        } catch {
            if ($commonStartMenu -and $linkPath.StartsWith($commonStartMenu.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) {
                $protectedStartMenu += $linkPath
            }
            # Public desktop/Start menu links often need administrator rights.
            # Keep the same ordinary name in this user's corresponding folder.
            if (!$personalCopies.ContainsKey($linkPath)) { continue }
            try {
                $personalPath = $personalCopies[$linkPath]
                $parent = Split-Path -LiteralPath $personalPath
                if (!(Test-Path -LiteralPath $parent)) { $null = New-Item -ItemType Directory -Path $parent -Force }
                if (!(Test-Path -LiteralPath $personalPath)) { Copy-Item -LiteralPath $linkPath -Destination $personalPath }
                $personalLink = $shell.CreateShortcut($personalPath)
                if ($executablePaths -contains $personalLink.TargetPath) {
                    $arguments = Add-ConnectionArguments $personalLink.Arguments
                    if ($personalLink.Arguments -cne $arguments) { $personalLink.Arguments = $arguments; $personalLink.Save() }
                    $launchEntries++
                }
            } catch { Write-Verbose "Could not configure personal shortcut for: $linkPath" }
        }
    }
}
# Elevation is allowed only by an explicit UI action. Background Restore keeps
# repairing per-user overrides without opening UAC prompts on every update.
if ($RepairCommonStartMenu -and $Action -notin @('Check', 'OpenStandard') -and $protectedStartMenu.Count -gt 0) {
    $repairScript = {
        $ErrorActionPreference = 'Stop'
        $failed = $false
        $shell = New-Object -ComObject WScript.Shell
        $root = [Environment]::GetFolderPath('CommonStartMenu')
        $backupRoot = Join-Path ([Environment]::GetFolderPath('CommonApplicationData')) ('Isle\NetEaseShortcutBackup\' + [guid]::NewGuid().ToString('N'))
        foreach ($file in Get-ChildItem -LiteralPath $root -Filter '*.lnk' -Recurse -ErrorAction SilentlyContinue) {
            try {
                $shortcut = $shell.CreateShortcut($file.FullName)
                if ([IO.Path]::GetFileName($shortcut.TargetPath) -ine 'cloudmusic.exe' -or !(Test-Path -LiteralPath $shortcut.TargetPath -PathType Leaf)) { continue }
                $arguments = $shortcut.Arguments -replace '(^|\s)--remote-debugging-port(?:=|\s+)\d+(?=\s|$)', ' '
                $arguments = $arguments -replace '(^|\s)--remote-debugging-address(?:=|\s+)\S+', ' '
                $arguments = ($arguments.Trim() + ' --remote-debugging-port=9223 --remote-debugging-address=127.0.0.1').Trim()
                if ($shortcut.Arguments -cne $arguments) {
                    $backup = Join-Path $backupRoot $file.FullName.Substring($root.Length).TrimStart('\')
                    $null = New-Item -ItemType Directory -Path (Split-Path -LiteralPath $backup) -Force
                    Copy-Item -LiteralPath $file.FullName -Destination $backup
                    $shortcut.Arguments = $arguments
                    $shortcut.Save()
                }
            } catch { $failed = $true }
        }
        if ($failed) { exit 1 }
    }
    try {
        # Pass fixed code directly to the elevated process, never execute an
        # administrator script from a user-writable temporary file.
        $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($repairScript.ToString()))
        $null = Start-Process -FilePath powershell.exe -Verb RunAs -WindowStyle Hidden -ArgumentList '-NoProfile', '-NonInteractive', '-EncodedCommand', $encoded -Wait -PassThru
    } catch { Write-Verbose 'Common Start menu repair was declined or unavailable' }
}
$pendingStartMenu = @($protectedStartMenu | Select-Object -Unique | Where-Object {
    try {
        $saved = $shell.CreateShortcut($_)
        $saved.Arguments -cne (Add-ConnectionArguments $saved.Arguments)
    } catch { $true }
})
$hasConnectionArguments = @($players | Where-Object {
    $_.ExecutablePath -ieq $exe -and
    $_.CommandLine -notmatch '(^|\s)--type(?:=|\s+)' -and
    $_.CommandLine -match '(^|\s)--remote-debugging-port(?:=|\s+)9223(\s|$)'
}).Count -gt 0
# Automatic restore never interrupts playback. A temporary CDP outage (or a
# player still starting) is not evidence that the process needs restarting.
$restartRequired = $Action -eq 'Connect' -and $players.Count -gt 0 -and !$hasConnectionArguments -and $env:ISLE_NETEASE_CONNECTED -ne '1'
if ($restartRequired) {
    foreach ($player in $players) {
        if ($player.ExecutablePath -ieq $exe) { Stop-Process -Id $player.ProcessId -Force -ErrorAction SilentlyContinue }
    }
    # Termination is asynchronous; allow child processes and the singleton
    # owner to exit before launching the replacement instance.
    $exitDeadline = [DateTime]::UtcNow.AddSeconds(5)
    do {
        $remaining = @(Get-CimInstance Win32_Process -Filter "name='cloudmusic.exe'" | Where-Object { $_.ExecutablePath -ieq $exe })
        if (!$remaining.Count) { break }
        if ([DateTime]::UtcNow -ge $exitDeadline) { throw 'NETEASE_RESTART_FAILED' }
        Start-Sleep -Milliseconds 100
    } while ($true)
}
if ($Action -eq 'OpenEnhanced' -or ($Action -eq 'Connect' -and ($players.Count -eq 0 -or $restartRequired))) {
    Start-Process -FilePath $exe -ArgumentList '--remote-debugging-port=9223', '--remote-debugging-address=127.0.0.1' -WorkingDirectory (Split-Path -LiteralPath $exe) -WindowStyle Hidden
}
$configured = $launchEntries -gt 0
if (Test-Path -LiteralPath $shortcutPath) {
    $savedLink = $shell.CreateShortcut($shortcutPath)
    $configured = $configured -or ($savedLink.TargetPath -ieq $exe -and $savedLink.Arguments -match '(^|\s)--remote-debugging-port=9223(\s|$)')
}
$watchPaths = @(@($exe) + $repairLinks + @($personalCopies.Values) + @($shortcutPath) + $userFolders + @($commonToPersonal.Keys) | Where-Object { $_ } | Select-Object -Unique | Select-Object -First 512)
@{ executablePath = $exe; shortcutPath = $(if ($launchEntries -eq 0 -and (Test-Path -LiteralPath $shortcutPath)) { $shortcutPath } else { '' }); configured = $configured; launchEntries = $launchEntries; pendingStartMenuEntries = $pendingStartMenu.Count; running = ($players.Count -gt 0 -or $Action -in @('Connect', 'OpenEnhanced')); restarted = $restartRequired; restartRequired = ($players.Count -gt 0 -and !$restartRequired -and !$hasConnectionArguments -and $env:ISLE_NETEASE_CONNECTED -ne '1'); watchPaths = $watchPaths } | ConvertTo-Json -Compress
