# Offline Start menu integration: real repair script, fake shortcuts and UAC.
$ErrorActionPreference = 'Stop'
$setupScript = Join-Path $PSScriptRoot '../src-tauri/windows/netease-connect.ps1'
function Test-StartMenuCase([string]$Case) {
    $exe = 'C:\NetEase\cloudmusic.exe'
    $commonRoot = [Environment]::GetFolderPath('CommonStartMenu')
    $userRoot = [Environment]::GetFolderPath('StartMenu')
    $commonLink = Join-Path $commonRoot 'Programs\NetEase\Music.lnk'
    $userLink = Join-Path $userRoot 'Programs\NetEase\Music.lnk'
    $ready = '--autostart --remote-debugging-port=9223 --remote-debugging-address=127.0.0.1'
    $state = @{ links = @{}; elevations = 0; saves = 0; copies = 0 }
    $state.links[$commonLink] = $(if ($Case -eq 'ready') { $ready } else { '--autostart --remote-debugging-port=9222' })
    $shell = [pscustomobject]@{ State=$state; Common=$commonLink; Exe=$exe; Writable=($Case -eq 'writable') }
    $shell | Add-Member ScriptMethod CreateShortcut {
        param($path)
        $shortcut = [pscustomobject]@{ State=$this.State; Key=$path; Protected=($path -eq $this.Common -and !$this.Writable); TargetPath=$this.Exe; Arguments=$this.State.links[$path]; WorkingDirectory='C:\NetEase'; IconLocation='C:\NetEase\cloudmusic.exe,0' }
        $shortcut | Add-Member ScriptMethod Save {
            if ($this.Protected) { throw 'access denied' }
            $this.State.links[$this.Key] = $this.Arguments
            $this.State.saves++
        }
        $shortcut
    }
    function Get-CimInstance { param($ClassName,$Filter); [pscustomobject]@{ExecutablePath=$exe;ProcessId=123;CommandLine='cloudmusic.exe --remote-debugging-port=9223'} }
    function New-Object { param($ComObject); if ($ComObject -ne 'WScript.Shell') { throw 'Unexpected COM access' }; $shell }
    function Test-Path { param($LiteralPath,$PathType); $LiteralPath -eq $exe -or $LiteralPath -in @($commonRoot,$userRoot,(Split-Path -LiteralPath $userLink)) -or $state.links.ContainsKey($LiteralPath) }
    function Get-ChildItem { param($LiteralPath,$Filter,[switch]$Recurse,$ErrorAction); if ($LiteralPath -eq $commonRoot) { [pscustomobject]@{FullName=$commonLink} } }
    function New-Item { param($ItemType,$Path,[switch]$Force); if ($ItemType -ne 'Directory' -or $Path -notlike '*Isle\NetEaseShortcutBackup\*') { throw 'Unexpected directory creation' } }
    function Copy-Item { param($LiteralPath,$Destination); if ($LiteralPath -ne $commonLink -or ($Destination -ne $userLink -and $Destination -notlike '*Isle\NetEaseShortcutBackup\*')) { throw 'Unexpected file copy' }; $state.links[$Destination]=$state.links[$LiteralPath]; $state.copies++ }
    function Stop-Process { throw 'Start menu maintenance must not stop the player' }
    function Start-Process {
        param($FilePath,$Verb,$WindowStyle,$ArgumentList,[switch]$Wait,[switch]$PassThru)
        if ($Case -notin @('elevated','declined') -or $FilePath -ne 'powershell.exe' -or $Verb -ne 'RunAs' -or $WindowStyle -ne 'Hidden' -or !$Wait -or !$PassThru) { throw 'Unexpected elevation or player launch' }
        $state.elevations++
        $code = [Text.Encoding]::Unicode.GetString([Convert]::FromBase64String($ArgumentList[-1]))
        if ($code -notmatch "GetFolderPath\('CommonStartMenu'\)" -or $code -notmatch 'NetEaseShortcutBackup' -or $code -match 'Stop-Process|HKCU:|HKLM:') { throw 'Elevated repair must be restricted to shared shortcuts' }
        if ($Case -eq 'declined') { throw 'UAC cancelled' }
        $shell.Writable=$true
        # Execute the actual elevated payload with fakes, without terminating
        # the regression process if its final failure branch is reached.
        & ([scriptblock]::Create($code.Replace('exit 1', "throw 'Elevated repair failed'")))
    }
    $options = @{ Action=$(if ($Case -eq 'check') { 'Check' } else { 'Restore' }) }
    if ($Case -in @('elevated','declined','ready','check')) { $options.RepairCommonStartMenu=$true }
    $result = & $setupScript @options | ConvertFrom-Json
    if ($Case -in @('elevated','declined')) {
        if ($state.elevations -ne 1) { throw 'Explicit action must attempt UAC only once' }
    } elseif ($state.elevations -ne 0) { throw 'Background/unchanged/check action opened UAC' }
    $pending = $(if ($Case -in @('protected','declined')) { 1 } else { 0 })
    if ($result.pendingStartMenuEntries -ne $pending) { throw "$Case reported incorrect pending entries" }
    if ($Case -eq 'check') {
        if ($state.saves -or $state.copies -or $state.links[$commonLink] -eq $ready) { throw 'Check must be read-only' }
    } elseif ($Case -in @('protected','declined','elevated')) {
        if ($state.links[$userLink] -ne $ready -or !$result.configured) { throw 'Personal Start menu fallback was not repaired' }
    }
    if ($Case -in @('writable','ready','elevated') -and $state.links[$commonLink] -ne $ready) { throw 'Shared Start menu was not repaired' }
    if ($Case -eq 'ready' -and $state.saves -ne 0) { throw 'Unchanged shortcuts must not be saved' }
    if ($result.watchPaths -notcontains $commonLink -or $result.watchPaths -notcontains $userLink) { throw 'Both Start menu entries must be watched for future repair' }
    "PASS Start menu $Case"
}
foreach ($case in @('writable','protected','elevated','declined','ready','check')) { Test-StartMenuCase $case }
