param([ValidateSet('Check', 'Connect', 'Restore')][string]$Action = 'Check')
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$players = @(Get-CimInstance Win32_Process -Filter "name='cloudmusic.exe'")
$candidates = @($players | ForEach-Object { $_.ExecutablePath })
$candidates += $env:ISLE_NETEASE_PATH
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
$commonToPersonal = @{}
foreach ($pair in @(@('CommonDesktopDirectory','Desktop'), @('CommonStartMenu','StartMenu'), @('CommonStartup','Startup'))) {
    $common = [Environment]::GetFolderPath($pair[0])
    $personal = [Environment]::GetFolderPath($pair[1])
    if ($common -and $personal) { $commonToPersonal[$common] = $personal }
}
foreach ($folder in @($userFolders + @([Environment]::GetFolderPath('CommonStartMenu'), [Environment]::GetFolderPath('CommonDesktopDirectory'), [Environment]::GetFolderPath('CommonStartup'))) | Select-Object -Unique) {
    if ($folder -and (Test-Path -LiteralPath $folder)) {
        foreach ($link in Get-ChildItem -LiteralPath $folder -Filter '*.lnk' -Recurse -ErrorAction SilentlyContinue) {
            $target = $shell.CreateShortcut($link.FullName).TargetPath
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
foreach ($root in @(${env:ProgramFiles}, ${env:ProgramFiles(x86)}, $env:LOCALAPPDATA)) {
    if ($root) { $candidates += (Join-Path $root 'NetEase\CloudMusic\cloudmusic.exe') }
}
$executablePaths = @($candidates | Where-Object { $_ -and (Test-Path -LiteralPath $_ -PathType Leaf) -and [IO.Path]::GetFileName($_) -ieq 'cloudmusic.exe' } | Select-Object -Unique)
$exe = $executablePaths | Select-Object -First 1
if (!$exe) { throw 'NETEASE_NOT_FOUND' }
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
        $shortcut.TargetPath = $exe
        $shortcut.Arguments = Add-ConnectionArguments ''
        $shortcut.WorkingDirectory = Split-Path -LiteralPath $exe
        $shortcut.IconLocation = "$exe,0"
        $shortcut.Save()
    }
    foreach ($linkPath in ($repairLinks | Select-Object -Unique)) {
        try {
            $existing = $shell.CreateShortcut($linkPath)
            if ($executablePaths -contains $existing.TargetPath) {
                $existing.Arguments = Add-ConnectionArguments $existing.Arguments
                $existing.Save()
                $launchEntries++
            }
        } catch {
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
                    $personalLink.Arguments = Add-ConnectionArguments $personalLink.Arguments
                    $personalLink.Save()
                    $launchEntries++
                }
            } catch { Write-Verbose "Could not configure personal shortcut for: $linkPath" }
        }
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
$configured = $launchEntries -gt 0
if (Test-Path -LiteralPath $shortcutPath) {
    $savedLink = $shell.CreateShortcut($shortcutPath)
    $configured = $configured -or ($savedLink.TargetPath -ieq $exe -and $savedLink.Arguments -match '(^|\s)--remote-debugging-port=9223(\s|$)')
}
@{ executablePath = $exe; shortcutPath = $(if ($launchEntries -eq 0 -and (Test-Path -LiteralPath $shortcutPath)) { $shortcutPath } else { '' }); configured = $configured; launchEntries = $launchEntries; running = ($players.Count -gt 0 -or $Action -eq 'Connect'); restarted = $restartRequired } | ConvertTo-Json -Compress
