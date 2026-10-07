param(
    [ValidateRange(1, 28800)][int]$Seconds = 20,
    [ValidateRange(0, 600)][int]$WarmupSeconds = 0,
    [ValidatePattern('^[a-zA-Z0-9_-]+$')][string]$Label = 'native-idle',
    [string]$OutputDirectory = 'dist/performance',
    [int]$RootPid = 0,
    [switch]$Control,
    [switch]$IncludeGpu
)
$ErrorActionPreference = 'Stop'
$logicalCores = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
$os = Get-CimInstance Win32_OperatingSystem
$gpuStatus = if ($IncludeGpu) { 'requested' } else { 'N/A: GPU counters not requested' }
$previous = @{}
$samples = [Collections.Generic.List[object]]::new()
$processHistory = @{}
$timer = [Diagnostics.Stopwatch]::StartNew()

function Read-Sample {
    $all = @(Get-CimInstance Win32_Process)
    $roots = @($all | Where-Object { $_.Name -eq 'isle.exe' -and ($RootPid -eq 0 -or $_.ProcessId -eq $RootPid) })
    if (-not $Control -and -not $roots.Count) { throw 'No matching running Isle process found.' }
    $ids = @($roots | ForEach-Object { [int]$_.ProcessId })
    do {
        $children = @($all | Where-Object { [int]$_.ParentProcessId -in $ids -and [int]$_.ProcessId -notin $ids })
        $ids += @($children | ForEach-Object { [int]$_.ProcessId })
    } while ($children.Count)
    $at = $timer.Elapsed.TotalSeconds
    $memory = @(Get-CimInstance Win32_PerfFormattedData_PerfProc_Process | Where-Object { [int]$_.IDProcess -in $ids })
    $gpu = @()
    $gpuMemory = @()
    if ($IncludeGpu) {
        try {
            $gpu = @(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine)
            $gpuMemory = @(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUProcessMemory)
            $script:gpuStatus = 'available: per-PID engines; engine percentages must not be summed as total GPU utilization'
        } catch { $script:gpuStatus = 'N/A: GPU performance counters unavailable' }
    }
    $rows = @(foreach ($item in $all | Where-Object { [int]$_.ProcessId -in $ids }) {
        $processId = [int]$item.ProcessId
        $process = Get-Process -Id $processId -ErrorAction SilentlyContinue
        if (-not $process) { continue }
        try {
            $cpu = $process.CPU
            $birth = $process.StartTime.ToUniversalTime().ToString('o')
            $key = "$processId/$birth"
            $cpuSingleCore = $null
            if ($previous.ContainsKey($key)) {
                $deltaSeconds = $at - $previous[$key].at
                if ($deltaSeconds -gt 0) { $cpuSingleCore = 100 * [math]::Max(0, $cpu - $previous[$key].cpu) / $deltaSeconds }
            }
            $previous[$key] = @{ at = $at; cpu = $cpu }
            $role = if ($item.Name -eq 'isle.exe') { 'app' } elseif ($item.Name -ne 'msedgewebview2.exe') { $item.Name } elseif ($item.CommandLine -match '--type=([^ ]+)') { $matches[1] } else { 'browser' }
            $counters = $memory | Where-Object { [int]$_.IDProcess -eq $processId } | Select-Object -First 1
            if (-not $processHistory.ContainsKey($key)) {
                $version = if ($item.ExecutablePath) { (Get-Item -LiteralPath $item.ExecutablePath).VersionInfo.ProductVersion } else { $null }
                $processHistory[$key] = [pscustomobject]@{ pid=$processId; parentPid=[int]$item.ParentProcessId; name=$item.Name; role=$role; version=$version; startedUtc=$birth; executable=$item.ExecutablePath }
            }
            $machineCpu = if ($null -ne $cpuSingleCore) { $cpuSingleCore / $logicalCores } else { $null }
            $gpuPattern = "^pid_${processId}_"
            [pscustomobject]@{
                pid = $processId
                role = $role
                cpuPercentOfMachine = $machineCpu
                cpuPercentOfOneCore = $cpuSingleCore
                cpuSampleAvailable = ($null -ne $cpuSingleCore)
                privateWorkingSetBytes = if ($counters) { $counters.WorkingSetPrivate } else { $null }
                workingSetBytes = $process.WorkingSet64
                privateBytes = $process.PrivateMemorySize64
                handles = $process.HandleCount
                threads = $process.Threads.Count
                gpuEngines = @($gpu | Where-Object Name -match $gpuPattern | ForEach-Object { [pscustomobject]@{ engine=$_.Name; utilizationPercent=$_.UtilizationPercentage } })
                gpuMemory = @($gpuMemory | Where-Object Name -match $gpuPattern | ForEach-Object { [pscustomobject]@{ instance=$_.Name; dedicatedBytes=$_.DedicatedUsage; sharedBytes=$_.SharedUsage; committedBytes=$_.TotalCommitted } })
            }
        } catch {
            # A process can exit between enumeration and reading its counters.
            if (Get-Process -Id $processId -ErrorAction SilentlyContinue) { throw }
        } finally { $process.Dispose() }
    })
    [pscustomobject]@{
        elapsedSeconds=$at
        timestampUtc=[DateTime]::UtcNow.ToString('o')
        machineCpuPercent=(Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor -Filter "Name='_Total'").PercentProcessorTime
        processCount=$rows.Count
        processes=$rows
        totalCpuPercentOfMachine=($rows | Measure-Object cpuPercentOfMachine -Sum).Sum
        totalPrivateWorkingSetBytes=($rows | Measure-Object privateWorkingSetBytes -Sum).Sum
        totalPrivateBytes=($rows | Measure-Object privateBytes -Sum).Sum
        totalHandles=($rows | Measure-Object handles -Sum).Sum
        totalThreads=($rows | Measure-Object threads -Sum).Sum
    }
}

if ($WarmupSeconds) { Start-Sleep -Seconds $WarmupSeconds }
$null = Read-Sample
$started = $timer.Elapsed.TotalSeconds
for ($index=0; $timer.Elapsed.TotalSeconds - $started -lt $Seconds; $index++) {
    $remaining = [math]::Min($started + $Seconds, $started + $index + 1) - $timer.Elapsed.TotalSeconds
    if ($remaining -gt 0) { Start-Sleep -Milliseconds ([int]($remaining * 1000)) }
    $samples.Add((Read-Sample))
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$report = [ordered]@{
    label=$Label
    createdUtc=[DateTime]::UtcNow.ToString('o')
    requestedSeconds=$Seconds
    actualSeconds=$timer.Elapsed.TotalSeconds - $started
    warmupSeconds=$WarmupSeconds
    logicalCores=$logicalCores
    os=$os.Caption
    osBuild=$os.BuildNumber
    control=[bool]$Control
    scope='Dynamic Isle descendant tree. Per-process CPU deltas keyed by PID and start time. First samples for new PIDs have unavailable CPU; scene, DPI and input corpus must be recorded separately. Sampling overhead also affects the control run.'
    gpuStatus=$gpuStatus
    unavailable=@('JS heap/DOM/retainers require WebView2 diagnostics','Actual frame presentation requires ETW/PresentMon','IPC bytes/in-flight and FFT counters require a profiling build')
    processHistory=@($processHistory.Values)
    samples=@($samples.ToArray())
}
$report | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $OutputDirectory "$Label.json") -Encoding utf8
Write-Output "Saved $Label ($($samples.Count) samples, dynamic process tree)."
