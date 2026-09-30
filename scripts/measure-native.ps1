param(
    [int]$Seconds = 20,
    [string]$Label = 'native-playing-compact',
    [string]$OutputDirectory = 'dist/performance'
)
$ErrorActionPreference = 'Stop'
if ($Seconds -lt 1 -or $Seconds -gt 60) { throw 'Sample duration must be between 1 and 60 seconds.' }
$roots = @(Get-CimInstance Win32_Process -Filter "Name = 'isle.exe'")
if (-not $roots.Count) { throw 'No running Isle process found.' }
$webviews = @(Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe'")
$ids = @($roots | ForEach-Object { [int]$_.ProcessId })
do {
    $children = @($webviews | Where-Object { [int]$_.ParentProcessId -in $ids -and [int]$_.ProcessId -notin $ids })
    $ids += @($children | ForEach-Object { [int]$_.ProcessId })
} while ($children.Count)
$processes = @($roots) + @($webviews | Where-Object { [int]$_.ProcessId -in $ids })
$logicalCores = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
$startCpu = @{}
foreach ($item in $processes) { $startCpu[[int]$item.ProcessId] = (Get-Process -Id $item.ProcessId).CPU }
$timer = [Diagnostics.Stopwatch]::StartNew()
Start-Sleep -Seconds $Seconds
$endCpu = @{}
foreach ($item in $processes) {
    $process = Get-Process -Id $item.ProcessId -ErrorAction SilentlyContinue
    if ($process) { $endCpu[[int]$item.ProcessId] = $process.CPU }
}
$elapsed = $timer.Elapsed.TotalSeconds
$memory = @(Get-CimInstance Win32_PerfFormattedData_PerfProc_Process | Where-Object { [int]$_.IDProcess -in $ids })
$rows = @(foreach ($item in $processes) {
    $processId = [int]$item.ProcessId
    if (-not $endCpu.ContainsKey($processId)) { continue }
    $role = if ($item.Name -eq 'isle.exe') { 'app' } elseif ($item.CommandLine -match '--type=([^ ]+)') { $matches[1] } else { 'browser' }
    $version = if ($item.Name -eq 'isle.exe' -and $item.ExecutablePath) { (Get-Item -LiteralPath $item.ExecutablePath).VersionInfo.ProductVersion } else { $null }
    $counters = $memory | Where-Object { [int]$_.IDProcess -eq $processId } | Select-Object -First 1
    [pscustomobject]@{
        pid = $processId
        parentPid = [int]$item.ParentProcessId
        role = $role
        version = $version
        cpuPercentOfMachine = 100 * ($endCpu[$processId] - $startCpu[$processId]) / $elapsed / $logicalCores
        privateWorkingSetBytes = $counters.WorkingSetPrivate
        workingSetBytes = $counters.WorkingSet
        privateBytes = $counters.PrivateBytes
    }
})
$report = [ordered]@{
    label = $Label
    sampleSeconds = $elapsed
    logicalCores = $logicalCores
    scope = 'Running Isle and its WebView2 process tree. CPU normalized to machine logical cores; ending private working set matches the Task Manager process memory concept. Scene supplied by user, not automated.'
    totalCpuPercentOfMachine = ($rows | Measure-Object cpuPercentOfMachine -Sum).Sum
    totalPrivateWorkingSetBytes = ($rows | Measure-Object privateWorkingSetBytes -Sum).Sum
    processes = $rows
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $OutputDirectory "$Label.json") -Encoding utf8
$rows | Select-Object role,version,@{n='CPUPercent';e={[math]::Round($_.cpuPercentOfMachine,3)}},@{n='PrivateWorkingSetMiB';e={[math]::Round($_.privateWorkingSetBytes / 1MB,2)}} | Format-Table
Write-Output ('Total CPU: {0:N3}%; Private working set: {1:N2} MiB' -f $report.totalCpuPercentOfMachine,($report.totalPrivateWorkingSetBytes / 1MB))
