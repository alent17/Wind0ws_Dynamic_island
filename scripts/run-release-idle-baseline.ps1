param(
    [string]$Executable = 'src-tauri/target/release/isle.exe',
    [string]$OutputDirectory = 'dist/performance/acceptance-2026-10-08',
    [int]$RootPid = 0
)
$ErrorActionPreference = 'Stop'
$resolvedExe = (Resolve-Path -LiteralPath $Executable).Path
if (Get-Process cargo,rustc -ErrorAction SilentlyContinue) { throw 'Build tasks are still running.' }
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
if ($RootPid) {
    $root = Get-Process -Id $RootPid
    if ($root.Path -ne $resolvedExe) { throw 'Root PID does not use the expected Release executable.' }
} else {
    if (Get-Process isle -ErrorAction SilentlyContinue) { throw 'Close existing Isle instances before running the baseline.' }
    $root = Start-Process -FilePath $resolvedExe -WindowStyle Hidden -PassThru
}
$root.Id | Set-Content -LiteralPath (Join-Path $OutputDirectory 'release-root-pid')
$manifest = [ordered]@{
    commit = (git rev-parse HEAD)
    startedUtc = [DateTime]::UtcNow.ToString('o')
    executable = $resolvedExe
    sha256 = (Get-FileHash -LiteralPath $resolvedExe -Algorithm SHA256).Hash
    fileVersion = (Get-Item -LiteralPath $resolvedExe).VersionInfo.ProductVersion
    rootPid = $root.Id
    profile = 'release; tauri/custom-protocol; no attached DevTools'
    scene = 'Paused NetEase session, compact main window, no floating window; existing user settings retained'
    requestedRounds = 3
    warmupSecondsPerRound = 300
    sampleSecondsPerRound = 600
    completedRounds = @()
}
$manifestPath = Join-Path $OutputDirectory 'idle-run-manifest.json'
function Save-Manifest { $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding utf8 }
Save-Manifest
for ($round=1; $round -le 3; $round++) {
    if (-not (Get-Process -Id $root.Id -ErrorAction SilentlyContinue)) { throw 'Release process exited before a round.' }
    if (Get-Process cargo,rustc -ErrorAction SilentlyContinue) { throw 'A build started during acceptance.' }
    $label = "release-idle-r$round"
    [ordered]@{ round=$round; phase='warming-and-sampling'; startedUtc=[DateTime]::UtcNow.ToString('o'); rootPid=$root.Id } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $OutputDirectory 'status.json') -Encoding utf8
    Write-Output "${label}: 300s warm-up + 600s sampling"
    & powershell -NoProfile -File scripts/measure-native.ps1 -RootPid $root.Id -Seconds 600 -WarmupSeconds 300 -Label $label -IncludeGpu -OutputDirectory $OutputDirectory
    if ($LASTEXITCODE -ne 0) { throw "Sampling failed: $label" }
    $manifest.completedRounds += $label
    Save-Manifest
    Write-Output "Completed $label"
}
[ordered]@{ phase='idle-completed'; completedUtc=[DateTime]::UtcNow.ToString('o'); rootPid=$root.Id } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $OutputDirectory 'status.json') -Encoding utf8
Write-Output 'All three Release idle rounds completed. Release remains running for media acceptance.'
