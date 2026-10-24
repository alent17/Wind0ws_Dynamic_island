param(
    [Parameter(Mandatory = $true)][int]$TargetPid,
    [Parameter(Mandatory = $true)][string]$OutputCsv,
    [int]$Seconds = 45,
    [string]$PresentMonExe = 'dist/performance/c1-2026-10-12/tools/PresentMon-2.6.0-x64.exe'
)
$ErrorActionPreference = 'Stop'
$resolvedExe = (Resolve-Path -LiteralPath $PresentMonExe).Path
$resolvedCsv = [System.IO.Path]::GetFullPath($OutputCsv)
$logPath = "$resolvedCsv.log"
$statusPath = "$resolvedCsv.status.json"
$startedUtc = [DateTime]::UtcNow.ToString('o')
$process = Get-Process -Id $TargetPid
$arguments = @('--process_id', "$TargetPid", '--output_file', $resolvedCsv, '--timed', "$Seconds", '--terminate_after_timed', '--no_console_stats', '--write_display_metadata', '--date_time')
"PresentMon $($arguments -join ' ')" | Set-Content -LiteralPath "$resolvedCsv.command.txt" -Encoding utf8
& $resolvedExe @arguments *> $logPath
$exitCode = $LASTEXITCODE
$captureLog = if (Test-Path -LiteralPath $logPath) { Get-Content -LiteralPath $logPath -Raw } else { '' }
$outputBytes = if (Test-Path -LiteralPath $resolvedCsv) { (Get-Item -LiteralPath $resolvedCsv).Length } else { 0 }
[ordered]@{
    targetPid = $TargetPid
    targetPath = $process.Path
    elevated = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    startedUtc = $startedUtc
    finishedUtc = [DateTime]::UtcNow.ToString('o')
    exitCode = $exitCode
    captureComplete = ($exitCode -eq 0 -and $captureLog -match 'Started recording\.' -and $captureLog -match 'Stopped recording\.' -and $outputBytes -gt 0)
    outputExists = Test-Path -LiteralPath $resolvedCsv
    outputBytes = $outputBytes
    logPath = $logPath
    note = 'Non-elevated fixed-PID capture; keep privilege level identical for the matched M6/C1 pair.'
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $statusPath -Encoding utf8
exit $exitCode
