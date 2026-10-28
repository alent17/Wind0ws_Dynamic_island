# Read-only capability probe. Run with Windows PowerShell 5.1, not pwsh.
param([ValidateRange(1,60)][int]$Seconds = 6)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$null = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager, Windows.Media.Control, ContentType = WindowsRuntime]
$null = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionMediaProperties, Windows.Media.Control, ContentType = WindowsRuntime]
$asTask = [System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {
    $_.Name -eq 'AsTask' -and $_.IsGenericMethod -and $_.GetGenericArguments().Count -eq 1 -and
    $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1'
} | Select-Object -First 1
function Await-WinRT($Operation, [Type]$ResultType) {
    $task = $asTask.MakeGenericMethod($ResultType).Invoke($null, @($Operation))
    if (!$task.Wait(3000)) { throw 'SMTC operation timed out' }
    return $task.Result
}
$manager = Await-WinRT ([Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager]::RequestAsync()) ([Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager])
$processes = @(Get-CimInstance Win32_Process -Filter "name='cloudmusic.exe'" | ForEach-Object {
    @{ pid = $_.ProcessId; hasDebugArgument = ($_.CommandLine -match '--remote-debugging-port'); commandLineReadable = ($null -ne $_.CommandLine); version = $(if ($_.ExecutablePath) { (Get-Item -LiteralPath $_.ExecutablePath).VersionInfo.FileVersion } else { $null }) }
})
$samples = @()
$timer = [Diagnostics.Stopwatch]::StartNew()
do {
    foreach ($session in $manager.GetSessions()) {
        $properties = Await-WinRT ($session.TryGetMediaPropertiesAsync()) ([Windows.Media.Control.GlobalSystemMediaTransportControlsSessionMediaProperties])
        $playback = $session.GetPlaybackInfo()
        $timeline = $session.GetTimelineProperties()
        $samples += @{
            elapsedMs = $timer.ElapsedMilliseconds; id = $session.SourceAppUserModelId
            title = $properties.Title; artist = $properties.Artist
            status = [string]$playback.PlaybackStatus
            positionMs = $timeline.Position.TotalMilliseconds; durationMs = $timeline.EndTime.TotalMilliseconds
            lastUpdated = [string]$timeline.LastUpdatedTime
            declared = @{ playPause = $playback.Controls.IsPlayPauseToggleEnabled; previous = $playback.Controls.IsPreviousEnabled; next = $playback.Controls.IsNextEnabled; seek = $playback.Controls.IsPlaybackPositionEnabled; shuffle = $playback.Controls.IsShuffleEnabled; repeat = $playback.Controls.IsRepeatEnabled }
            shuffle = $playback.IsShuffleActive; repeat = [string]$playback.AutoRepeatMode
        }
    }
    Start-Sleep -Milliseconds 500
} while ($timer.Elapsed.TotalSeconds -lt $Seconds)
@{ observedAt = [DateTimeOffset]::Now.ToString('o'); readOnly = $true; processes = $processes; samples = $samples } | ConvertTo-Json -Depth 6
