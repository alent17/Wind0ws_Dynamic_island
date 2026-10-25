# Offline registry/launch-entry regression tests. No real registry/process writes.
$ErrorActionPreference = 'Stop'
$setupScript = Join-Path $PSScriptRoot '../src-tauri/windows/netease-connect.ps1'
$fakeExe = 'C:\Music Player\cloudmusic.exe'
$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$registry = @{}
$registry[$runKey] = @{ CloudMusic = ('"' + $fakeExe + '" --autostart'); Other = '"C:\Other.exe" --keep' }
$registry['Registry::HKEY_CLASSES_ROOT\orpheus\shell\open\command'] = @{ '' = ('"' + $fakeExe + '"--webapp "%1"') }
$registry['HKCU:\Software\Classes\orpheus\shell\open\command'] = @{ '' = ('"' + $fakeExe + '"--webapp "%1"'); OtherMetadata = 'keep' }
$registry['Registry::HKEY_CLASSES_ROOT\cloudmusic.mp3\shell\open\command'] = @{ '' = ('"' + $fakeExe + '" "%1" --remote-debugging-port=9222') }
$registry['Registry::HKEY_CLASSES_ROOT\cloudmusic.flac\shell\open\command'] = @{ '' = ('"C:\Other.exe" "' + $fakeExe + '"') }
$writes = [Collections.Generic.List[string]]::new()
function Get-CimInstance { param($ClassName, $Filter); [pscustomobject]@{ExecutablePath=$fakeExe;ProcessId=123;CommandLine=('"' + $fakeExe + '" --remote-debugging-port=9223')} }
function Test-Path { param($LiteralPath, $PathType); $LiteralPath -eq $fakeExe -or $registry.ContainsKey($LiteralPath) }
function Get-Item {
    param($LiteralPath)
    $lookup = $LiteralPath -replace '^Registry::HKEY_CLASSES_ROOT\\', 'HKCU:\Software\Classes\'
    if (!$registry.ContainsKey($lookup)) { $lookup = $LiteralPath }
    if (!$registry.ContainsKey($lookup)) { throw "Unexpected registry read: $LiteralPath" }
    $item = [pscustomobject]@{ Data = $registry[$lookup] }
    $item | Add-Member ScriptMethod GetValue { param($Name); $this.Data[$Name] }
    $item | Add-Member ScriptMethod GetValueNames { @($this.Data.Keys) }
    $item
}
function New-Item { param($Path, [switch]$Force); if ($registry.ContainsKey($Path)) { throw 'Existing registry keys must not be recreated' }; $registry[$Path] = @{}; $writes.Add($Path) }
function New-ItemProperty { param($LiteralPath, $Name, $Value, $PropertyType, [switch]$Force); $registry[$LiteralPath][$Name] = $Value; $writes.Add($LiteralPath + '|' + $Name) }
function Set-Item { param($LiteralPath, $Value); $registry[$LiteralPath][''] = $Value; $writes.Add($LiteralPath) }
function New-Object {
    param($ComObject)
    if ($ComObject -ne 'WScript.Shell') { throw 'Unexpected COM object' }
    $shell = [pscustomobject]@{}
    $shell | Add-Member ScriptMethod CreateShortcut { throw 'Normal launch entries must not require a new shortcut' }
    $shell
}
function Stop-Process { throw 'Launch entry repair must not stop the player' }
function Start-Process { throw 'Launch entry repair must not launch another player' }

$check = & $setupScript -Action Check | ConvertFrom-Json
if ($writes.Count -ne 0 -or $check.configured) { throw 'Check modified launch entries' }
$result = & $setupScript -Action Restore | ConvertFrom-Json
if (!$result.configured -or $result.launchEntries -ne 3 -or $result.shortcutPath) { throw 'Normal launch entry setup failed' }
foreach ($command in @($registry[$runKey].CloudMusic, $registry['HKCU:\Software\Classes\orpheus\shell\open\command'][''], $registry['HKCU:\Software\Classes\cloudmusic.mp3\shell\open\command'][''])) {
    if ([regex]::Matches($command, '--remote-debugging-port=9223').Count -ne 1 -or $command -notmatch '--remote-debugging-address=127.0.0.1') { throw 'Missing or duplicate connection arguments' }
}
if ($registry[$runKey].CloudMusic -notmatch '--autostart' -or $registry[$runKey].Other -ne '"C:\Other.exe" --keep') { throw 'Autostart preferences were changed' }
if ($registry['HKCU:\Software\Classes\orpheus\shell\open\command'][''] -notmatch '--webapp "%1"' -or $registry['HKCU:\Software\Classes\cloudmusic.mp3\shell\open\command'][''] -notmatch '"%1"' -or $registry.ContainsKey('HKCU:\Software\Classes\cloudmusic.flac\shell\open\command')) { throw 'Unrelated associations or placeholders changed' }
$backup = $registry['HKCU:\Software\Isle\NeteaseLaunchBackup']
if ($backup.Count -ne 3) { throw 'Original launch entries were not backed up' }
if ($registry['HKCU:\Software\Classes\orpheus\shell\open\command'].OtherMetadata -ne 'keep') { throw 'Existing registry metadata was overwritten' }
$writeCount = $writes.Count
$null = & $setupScript -Action Restore | ConvertFrom-Json
if ($writes.Count -ne $writeCount) { throw 'Repeated setup rewrote an already configured registry entry' }
$check = & $setupScript -Action Check | ConvertFrom-Json
if (!$check.configured -or $writes.Count -ne $writeCount) { throw 'Configured detection is not read-only' }
'PASS normal shortcuts are optional; autostart, URLs and files preserve arguments and backups'
'PASS unrelated applications stay unchanged; repeated setup is idempotent; no process restart'
