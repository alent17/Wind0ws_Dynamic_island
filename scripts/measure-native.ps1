param(
    [ValidateRange(1, 28800)][int]$Seconds = 20,
    [ValidateRange(0, 600)][int]$WarmupSeconds = 0,
    [ValidatePattern('^[a-zA-Z0-9_-]+$')][string]$Label = 'native-idle',
    [string]$OutputDirectory = 'dist/performance',
    [int]$RootPid = 0,
    [switch]$Control,
    [switch]$IncludeGpu,
    [string]$StopFile = '',
    [string]$RunId = '',
    [string]$ReadyFile = ''
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
    $sampleReadStartedUtc = [DateTime]::UtcNow.ToString('o')
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
            $cpuReadStartedUtc = [DateTime]::UtcNow.ToString('o')
            $cpu = [double]$process.TotalProcessorTime.TotalSeconds
            $cpuReadEndedUtc = [DateTime]::UtcNow.ToString('o')
            $counterAt = $timer.Elapsed.TotalSeconds
            $birth = $process.StartTime.ToUniversalTime().ToString('o')
            $key = "$processId/$birth"
            $cpuSingleCore = $null
            $prior = $null
            if ($previous.ContainsKey($key)) {
                $prior = $previous[$key]
                $deltaSeconds = $counterAt - $previous[$key].at
                if ($deltaSeconds -gt 0) { $cpuSingleCore = 100 * [math]::Max([double]0, [double]($cpu - $previous[$key].cpu)) / $deltaSeconds }
            }
            $previous[$key] = @{ at = $counterAt; cpu = $cpu; readStartedUtc = $cpuReadStartedUtc; readEndedUtc = $cpuReadEndedUtc }
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
                cumulativeCpuSeconds = $cpu
                processKey = $key
                cpuReadStartedUtc = $cpuReadStartedUtc
                cpuReadEndedUtc = $cpuReadEndedUtc
                previousCpuReadStartedUtc = if ($prior) { $prior.readStartedUtc } else { $null }
                previousCpuReadEndedUtc = if ($prior) { $prior.readEndedUtc } else { $null }
                previousCumulativeCpuSeconds = if ($prior) { $prior.cpu } else { $null }
                cpuDeltaSeconds = if ($prior) { $cpu - $prior.cpu } else { $null }
                cpuCounterTimestampSeconds = $counterAt
                cpuPercentOfMachine = $machineCpu
                cpuPercentOfOneCore = $cpuSingleCore
                cpuSampleAvailable = ($null -ne $cpuSingleCore)
                privateWorkingSetBytes = if ($counters) { $counters.WorkingSetPrivate } else { $null }
                workingSetBytes = $process.WorkingSet64
                privateBytes = $process.PrivateMemorySize64
                peakPagefileUsageBytes = $process.PeakPagedMemorySize64
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
        sampleReadStartedUtc=$sampleReadStartedUtc
        sampleReadEndedUtc=[DateTime]::UtcNow.ToString('o')
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

New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
if ($ReadyFile -and (-not $RunId -or $RootPid -le 0)) { throw 'ReadyFile requires RunId and explicit RootPid.' }
if ($ReadyFile -and (Test-Path -LiteralPath $ReadyFile)) { throw 'Refusing pre-existing readiness file.' }
$probeStartedUtc = [DateTime]::UtcNow.ToString('o')
if ($WarmupSeconds) { Start-Sleep -Seconds $WarmupSeconds }
$baselineSample = Read-Sample
$samples.Add($baselineSample)
$streamPath = Join-Path $OutputDirectory "$Label.samples.jsonl"
$baselineSample | ConvertTo-Json -Depth 12 -Compress | Set-Content -LiteralPath $streamPath -Encoding utf8
$readiness = $null
$started = $timer.Elapsed.TotalSeconds
for ($index=0; $timer.Elapsed.TotalSeconds - $started -lt $Seconds; $index++) {
    if ($StopFile -and (Test-Path -LiteralPath $StopFile)) { break }
    $remaining = [math]::Min($started + $Seconds, $started + $index + 1) - $timer.Elapsed.TotalSeconds
    if ($remaining -gt 0) { Start-Sleep -Milliseconds ([int]($remaining * 1000)) }
    $sample = Read-Sample
    $samples.Add($sample)
    $sample | ConvertTo-Json -Depth 12 -Compress | Add-Content -LiteralPath $streamPath -Encoding utf8
    if (-not $readiness -and $sample.processes.Count -gt 0 -and @($sample.processes | Where-Object { -not $_.cpuSampleAvailable }).Count -eq 0 -and $RootPid -in $sample.processes.pid) {
        $readiness = [ordered]@{ runId=$RunId; rootPid=$RootPid; probeStartedUtc=$probeStartedUtc; readyUtc=[DateTime]::UtcNow.ToString('o'); validIncrementSampleIndex=$samples.Count-1; processKeys=@($sample.processes.processKey); lastCpuReadUtc=($sample.processes.cpuReadEndedUtc | Sort-Object | Select-Object -Last 1) }
        if ($ReadyFile) {
            $readiness | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath "$ReadyFile.tmp" -Encoding utf8
            Move-Item -LiteralPath "$ReadyFile.tmp" -Destination $ReadyFile
        }
    }
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$report = [ordered]@{
    label=$Label
    runId=$RunId
    rootPid=$RootPid
    probeStartedUtc=$probeStartedUtc
    readiness=$readiness
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
