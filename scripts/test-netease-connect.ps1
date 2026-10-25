# Offline tests: all process, shortcut and discovery APIs are replaced with fakes.
$ErrorActionPreference = 'Stop'
$setupScript = Join-Path $PSScriptRoot '../src-tauri/windows/netease-connect.ps1'
function Test-SetupCase([string]$Case) {
    $state = @{ saved = $false; stopped = $false; launched = $false }
    $fakeExe = 'C:\Program Files\NetEase\CloudMusic\cloudmusic.exe'
    $link = [pscustomobject]@{ State = $state; FailSave = ($Case -eq 'saveFailure'); TargetPath = ''; Arguments = ''; WorkingDirectory = ''; IconLocation = '' }
    $link | Add-Member ScriptMethod Save { if ($this.FailSave) { throw 'fake save failure' }; $this.State.saved = $true }
    $fakeDesktop = [Environment]::GetFolderPath('Desktop')
    $fakeCommonDesktop = [Environment]::GetFolderPath('CommonDesktopDirectory')
    $originalFolder = $(if ($Case -eq 'protectedShortcut') { $fakeCommonDesktop } else { $fakeDesktop })
    $fakeOriginalPath = Join-Path $originalFolder 'Original NetEase.lnk'
    $fakePersonalPath = Join-Path $fakeDesktop 'Original NetEase.lnk'
    $existingLink = [pscustomobject]@{ State = $state; Protected = ($Case -eq 'protectedShortcut'); TargetPath = $fakeExe; Arguments = '--autostart --remote-debugging-port=9222' }
    $existingLink | Add-Member ScriptMethod Save { if ($this.Protected) { throw 'access denied' }; $this.State.repaired = $true; $this.State.saved = $true }
    $personalLink = [pscustomobject]@{ State = $state; TargetPath = $fakeExe; Arguments = '--autostart' }
    $personalLink | Add-Member ScriptMethod Save { $this.State.repaired = $true; $this.State.saved = $true }
    $shell = [pscustomobject]@{ Link = $link; Existing = $existingLink; ExistingPath = $fakeOriginalPath; Personal = $personalLink; PersonalPath = $fakePersonalPath }
    $shell | Add-Member ScriptMethod CreateShortcut { param($path); if ($path -eq $this.ExistingPath) { $this.Existing } elseif ($path -eq $this.PersonalPath) { $this.Personal } else { $this.Link } }
    function Get-CimInstance { param($ClassName, $Filter); if (!$state.stopped -and $Case -notin @('notFound', 'restoreAbsent', 'connectAbsent')) { [pscustomobject]@{ ExecutablePath = $fakeExe; ProcessId = 123; CommandLine = $(if ($Case -in @('restoreReady', 'connectReady')) { 'cloudmusic.exe --remote-debugging-port=9223' } else { 'cloudmusic.exe' }) } } }
    function Test-Path { param($LiteralPath, $PathType); $Case -ne 'notFound' -and ($LiteralPath -eq $fakeExe -or ($state.saved -and $LiteralPath -like '*.lnk') -or ($Case -eq 'repairShortcut' -and $LiteralPath -eq $fakeDesktop) -or ($Case -eq 'protectedShortcut' -and $LiteralPath -in @($fakeCommonDesktop,$fakeDesktop)) -or ($state.copied -and $LiteralPath -eq $fakePersonalPath)) }
    function Get-ChildItem { param($LiteralPath, $Filter, [switch]$Recurse, $ErrorAction); if (($Case -eq 'repairShortcut' -and $LiteralPath -eq $fakeDesktop) -or ($Case -eq 'protectedShortcut' -and $LiteralPath -eq $fakeCommonDesktop)) { [pscustomobject]@{ FullName = $fakeOriginalPath } } }
    function Copy-Item { param($LiteralPath, $Destination); if ($Case -ne 'protectedShortcut' -or $LiteralPath -ne $fakeOriginalPath -or $Destination -ne $fakePersonalPath) { throw 'unexpected copy' }; $state.copied = $true }
    function New-Object { param($ComObject); if ($ComObject -ne 'WScript.Shell') { throw 'unexpected COM object' }; $shell }
    function Stop-Process { param($Id, [switch]$Force, $ErrorAction); if (!$state.saved -or $Id -ne 123) { throw 'unsafe stop' }; $state.stopped = $true }
    function Start-Process { param($FilePath, $ArgumentList, $WorkingDirectory, $WindowStyle); if ($FilePath -ne $fakeExe -or (!$state.stopped -and $Case -ne 'connectAbsent') -or $ArgumentList[0] -ne '--remote-debugging-port=9223') { throw 'invalid launch' }; $state.launched = $true }
    $failed = $false
    $previousConnected = $env:ISLE_NETEASE_CONNECTED
    $env:ISLE_NETEASE_CONNECTED = $(if ($Case -in @('restoreConnected', 'connectConnected')) { '1' } else { '0' })
    try { $result = & $setupScript -Action $(if ($Case -eq 'check') { 'Check' } elseif ($Case -like 'restore*' -or $Case -eq 'protectedShortcut') { 'Restore' } else { 'Connect' }) | ConvertFrom-Json } catch { $failed = $true } finally { $env:ISLE_NETEASE_CONNECTED = $previousConnected }
    if ($Case -in @('notFound', 'saveFailure')) {
        if (!$failed -or $state.stopped -or $state.launched) { throw "$Case interrupted the player" }
    } elseif ($Case -eq 'check') {
        if ($failed -or $state.saved -or $state.stopped -or $state.launched -or $result.executablePath -ne $fakeExe) { throw 'Check must be read-only' }
    } elseif ($Case -in @('restoreReady', 'restoreAbsent', 'restoreConnected', 'restoreMissingArguments', 'connectReady', 'connectConnected', 'protectedShortcut')) {
        if ($failed -or !$state.saved -or $state.stopped -or $state.launched) { throw "$Case must not restart or launch the player" }
    } elseif ($Case -eq 'connectAbsent') {
        if ($failed -or !$state.saved -or $state.stopped -or !$state.launched) { throw 'Absent player should be launched without a restart' }
    } else {
        if ($failed -or !$state.saved -or !$state.stopped -or !$state.launched -or !$result.configured -or ($Case -ne 'repairShortcut' -and $link.TargetPath -ne $fakeExe)) { throw 'Connect failed' }
    }
    if ($Case -eq 'repairShortcut' -and (!$state.repaired -or $existingLink.Arguments -notmatch '--autostart' -or $existingLink.Arguments -match '9222' -or [regex]::Matches($existingLink.Arguments, '--remote-debugging-port=').Count -ne 1)) { throw 'Original shortcut arguments were not preserved and repaired' }
    if ($Case -eq 'protectedShortcut' -and (!$state.copied -or !$state.repaired -or $personalLink.Arguments -notmatch '--autostart' -or $personalLink.Arguments -notmatch '--remote-debugging-port=9223')) { throw 'Protected shortcut did not get a configured ordinary personal copy' }
    "PASS $Case"
}
foreach ($case in @('check', 'connect', 'notFound', 'saveFailure', 'restoreReady', 'restoreAbsent', 'restoreConnected', 'restoreMissingArguments', 'connectReady', 'connectConnected', 'connectAbsent', 'repairShortcut', 'protectedShortcut')) { Test-SetupCase $case }
