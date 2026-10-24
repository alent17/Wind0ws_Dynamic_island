# Offline tests: all process, shortcut and discovery APIs are replaced with fakes.
$ErrorActionPreference = 'Stop'
$setupScript = Join-Path $PSScriptRoot '../src-tauri/windows/netease-connect.ps1'
function Test-SetupCase([string]$Case) {
    $state = @{ saved = $false; stopped = $false; launched = $false }
    $fakeExe = 'C:\Program Files\NetEase\CloudMusic\cloudmusic.exe'
    $link = [pscustomobject]@{ State = $state; FailSave = ($Case -eq 'saveFailure'); TargetPath = ''; Arguments = ''; WorkingDirectory = ''; IconLocation = '' }
    $link | Add-Member ScriptMethod Save { if ($this.FailSave) { throw 'fake save failure' }; $this.State.saved = $true }
    $fakeDesktop = [Environment]::GetFolderPath('Desktop')
    $fakeOriginalPath = Join-Path $fakeDesktop 'Original NetEase.lnk'
    $existingLink = [pscustomobject]@{ State = $state; TargetPath = $fakeExe; Arguments = '--autostart --remote-debugging-port=9222' }
    $existingLink | Add-Member ScriptMethod Save { $this.State.repaired = $true }
    $shell = [pscustomobject]@{ Link = $link; Existing = $existingLink; ExistingPath = $fakeOriginalPath }
    $shell | Add-Member ScriptMethod CreateShortcut { param($path); if ($path -eq $this.ExistingPath) { $this.Existing } else { $this.Link } }
    function Get-CimInstance { param($ClassName, $Filter); if (!$state.stopped -and $Case -notin @('notFound', 'restoreAbsent', 'connectAbsent')) { [pscustomobject]@{ ExecutablePath = $fakeExe; ProcessId = 123; CommandLine = $(if ($Case -in @('restoreReady', 'connectReady')) { 'cloudmusic.exe --remote-debugging-port=9223' } else { 'cloudmusic.exe' }) } } }
    function Test-Path { param($LiteralPath, $PathType); $Case -ne 'notFound' -and ($LiteralPath -eq $fakeExe -or ($state.saved -and $LiteralPath -like '*.lnk') -or ($Case -eq 'repairShortcut' -and $LiteralPath -eq $fakeDesktop)) }
    function Get-ChildItem { param($LiteralPath, $Filter, [switch]$Recurse, $ErrorAction); if ($Case -eq 'repairShortcut' -and $LiteralPath -eq $fakeDesktop) { [pscustomobject]@{ FullName = $fakeOriginalPath } } }
    function New-Object { param($ComObject); if ($ComObject -ne 'WScript.Shell') { throw 'unexpected COM object' }; $shell }
    function Stop-Process { param($Id, [switch]$Force, $ErrorAction); if (!$state.saved -or $Id -ne 123) { throw 'unsafe stop' }; $state.stopped = $true }
    function Start-Process { param($FilePath, $ArgumentList, $WorkingDirectory, $WindowStyle); if ($FilePath -ne $fakeExe -or (!$state.stopped -and $Case -ne 'connectAbsent') -or $ArgumentList[0] -ne '--remote-debugging-port=9223') { throw 'invalid launch' }; $state.launched = $true }
    $failed = $false
    $previousConnected = $env:ISLE_NETEASE_CONNECTED
    $env:ISLE_NETEASE_CONNECTED = $(if ($Case -in @('restoreConnected', 'connectConnected')) { '1' } else { '0' })
    try { $result = & $setupScript -Action $(if ($Case -eq 'check') { 'Check' } elseif ($Case -like 'restore*') { 'Restore' } else { 'Connect' }) | ConvertFrom-Json } catch { $failed = $true } finally { $env:ISLE_NETEASE_CONNECTED = $previousConnected }
    if ($Case -in @('notFound', 'saveFailure')) {
        if (!$failed -or $state.stopped -or $state.launched) { throw "$Case interrupted the player" }
    } elseif ($Case -eq 'check') {
        if ($failed -or $state.saved -or $state.stopped -or $state.launched -or $result.executablePath -ne $fakeExe) { throw 'Check must be read-only' }
    } elseif ($Case -in @('restoreReady', 'restoreAbsent', 'restoreConnected', 'restoreMissingArguments', 'connectReady', 'connectConnected')) {
        if ($failed -or !$state.saved -or $state.stopped -or $state.launched) { throw "$Case must not restart or launch the player" }
    } elseif ($Case -eq 'connectAbsent') {
        if ($failed -or !$state.saved -or $state.stopped -or !$state.launched) { throw 'Absent player should be launched without a restart' }
    } else {
        if ($failed -or !$state.saved -or !$state.stopped -or !$state.launched -or !$result.configured -or $link.TargetPath -ne $fakeExe) { throw 'Connect failed' }
    }
    if ($Case -eq 'repairShortcut' -and (!$state.repaired -or $existingLink.Arguments -notmatch '--autostart' -or $existingLink.Arguments -match '9222' -or [regex]::Matches($existingLink.Arguments, '--remote-debugging-port=').Count -ne 1)) { throw 'Original shortcut arguments were not preserved and repaired' }
    "PASS $Case"
}
foreach ($case in @('check', 'connect', 'notFound', 'saveFailure', 'restoreReady', 'restoreAbsent', 'restoreConnected', 'restoreMissingArguments', 'connectReady', 'connectConnected', 'connectAbsent', 'repairShortcut')) { Test-SetupCase $case }
