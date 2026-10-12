param(
    [Parameter(Mandatory = $true)][string]$OutputCsv,
    [int]$TargetPid = 0,
    [string]$TargetProcessName = 'msedgewebview2.exe',
    [string]$PresentMonExe = 'dist/performance/c1-2026-10-12/tools/PresentMon-2.6.0-x64.exe',
    [int]$Seconds = 30
)
$ErrorActionPreference = 'Stop'
$resolvedExe = (Resolve-Path -LiteralPath $PresentMonExe).Path
$resolvedCsv = [System.IO.Path]::GetFullPath($OutputCsv)
$logPath = "$resolvedCsv.log"
$statusPath = "$resolvedCsv.status.json"
$startedUtc = [DateTime]::UtcNow.ToString('o')
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    $scriptPath = (Resolve-Path -LiteralPath $PSCommandPath).Path
    $forwardArgs = @(
        '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$scriptPath`"",
        '-TargetPid', "$TargetPid", '-TargetProcessName', "`"$TargetProcessName`"", '-OutputCsv', "`"$resolvedCsv`"",
        '-PresentMonExe', "`"$resolvedExe`"", '-Seconds', "$Seconds"
    )
    $elevated = Start-Process -FilePath 'powershell.exe' -ArgumentList $forwardArgs -Verb RunAs -WindowStyle Hidden -Wait -PassThru
    exit $elevated.ExitCode
}

$processPath = if ($TargetPid -gt 0) { (Get-Process -Id $TargetPid).Path } else { $TargetProcessName }
$targetArgs = if ($TargetPid -gt 0) { @('--process_id', "$TargetPid") } else { @('--process_name', $TargetProcessName) }
$arguments = @($targetArgs + @(
    '--output_file', $resolvedCsv,
    '--timed', "$Seconds",
    '--terminate_after_timed',
    '--no_console_stats'
))
"PresentMon $($arguments -join ' ')" | Set-Content -LiteralPath "$resolvedCsv.command.txt" -Encoding utf8
& $resolvedExe @arguments *> $logPath
$exitCode = $LASTEXITCODE
[ordered]@{
    targetPid = if ($TargetPid -gt 0) { $TargetPid } else { $null }
    targetProcessName = if ($TargetPid -gt 0) { $null } else { $TargetProcessName }
    targetPath = $processPath
    elevated = $isAdmin
    presentMonVersion = (& $resolvedExe --version 2>&1 | Select-Object -First 1)
    startedUtc = $startedUtc
    finishedUtc = [DateTime]::UtcNow.ToString('o')
    exitCode = $exitCode
    outputExists = Test-Path -LiteralPath $resolvedCsv
    outputBytes = if (Test-Path -LiteralPath $resolvedCsv) { (Get-Item -LiteralPath $resolvedCsv).Length } else { 0 }
    logPath = $logPath
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $statusPath -Encoding utf8
exit $exitCode

