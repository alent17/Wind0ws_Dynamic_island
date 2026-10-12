param(
    [Parameter(Mandatory = $true)][int]$RootPid,
    [Parameter(Mandatory = $true)][string]$OutputPath,
    [int]$Seconds = 15
)

$samples = [System.Collections.Generic.List[object]]::new()
$clock = [System.Diagnostics.Stopwatch]::StartNew()
while ($clock.Elapsed.TotalSeconds -lt $Seconds) {
    $processes = @(Get-CimInstance Win32_Process)
    $ids = [System.Collections.Generic.HashSet[int]]::new()
    [void]$ids.Add($RootPid)
    do {
        $changed = $false
        foreach ($process in $processes) {
            if ($ids.Contains([int]$process.ParentProcessId) -and $ids.Add([int]$process.ProcessId)) {
                $changed = $true
            }
        }
    } while ($changed)

    $threadCounts = @(
        Get-Process -Id @($ids) -ErrorAction SilentlyContinue |
            ForEach-Object { [int]$_.Threads.Count }
    )
    $samples.Add([pscustomobject]@{
        elapsedMs = [math]::Round($clock.Elapsed.TotalMilliseconds, 1)
        processCount = $threadCounts.Count
        threadCount = ($threadCounts | Measure-Object -Sum).Sum
        largestProcessThreadCount = ($threadCounts | Measure-Object -Maximum).Maximum
    })
    Start-Sleep -Milliseconds 250
}

$parent = Split-Path -Parent $OutputPath
if ($parent) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
$samples | Export-Csv -LiteralPath $OutputPath -NoTypeInformation
$peak = $samples | Measure-Object -Property threadCount -Maximum
"samples=$($samples.Count) peakThreads=$($peak.Maximum) rootPid=$RootPid output=$OutputPath"

