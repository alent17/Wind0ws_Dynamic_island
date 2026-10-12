param(
    [Parameter(Mandatory = $true)][int]$ProcessId,
    [Parameter(Mandatory = $true)][string]$OutputPath,
    [int]$Seconds = 15,
    [int]$SampleIntervalMs = 5
)
$ErrorActionPreference = 'Stop'

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Sequential, Pack = 1)]
public struct UnsignedRatio { public uint Numerator; public uint Denominator; }

[StructLayout(LayoutKind.Sequential, Pack = 1)]
public struct DwmTimingInfo {
    public uint cbSize;
    public UnsignedRatio rateRefresh;
    public ulong qpcRefreshPeriod;
    public UnsignedRatio rateCompose;
    public ulong qpcVBlank;
    public ulong cRefresh;
    public uint cDXRefresh;
    public ulong qpcCompose;
    public ulong cFrame;
    public uint cDXPresent;
    public ulong cRefreshFrame;
    public ulong cFrameSubmitted;
    public uint cDXPresentSubmitted;
    public ulong cFrameConfirmed;
    public uint cDXPresentConfirmed;
    public ulong cRefreshConfirmed;
    public uint cDXRefreshConfirmed;
    public ulong cFramesLate;
    public uint cFramesOutstanding;
    public ulong cFrameDisplayed;
    public ulong qpcFrameDisplayed;
    public ulong cRefreshFrameDisplayed;
    public ulong cFrameComplete;
    public ulong qpcFrameComplete;
    public ulong cFramePending;
    public ulong qpcFramePending;
    public ulong cFramesDisplayed;
    public ulong cFramesComplete;
    public ulong cFramesPending;
    public ulong cFramesAvailable;
    public ulong cFramesDropped;
    public ulong cFramesMissed;
    public ulong cRefreshNextDisplayed;
    public ulong cRefreshNextPresented;
    public ulong cRefreshesDisplayed;
    public ulong cRefreshesPresented;
    public ulong cRefreshStarted;
    public ulong cPixelsReceived;
    public ulong cPixelsDrawn;
    public ulong cBuffersEmpty;
}

public static class DwmTiming {
    [DllImport("dwmapi.dll")]
    public static extern int DwmGetCompositionTimingInfo(IntPtr hwnd, ref DwmTimingInfo info);
}
'@

Get-Process -Id $ProcessId | Out-Null
$hwnd = [IntPtr]::Zero

$rows = [System.Collections.Generic.List[object]]::new()
$clock = [System.Diagnostics.Stopwatch]::StartNew()
$frequency = [double][System.Diagnostics.Stopwatch]::Frequency
$lastFrame = [uint64]::MaxValue
$lastQpc = [uint64]0
$infoSize = [Runtime.InteropServices.Marshal]::SizeOf([type][DwmTimingInfo])

while ($clock.Elapsed.TotalSeconds -lt $Seconds) {
    $info = [DwmTimingInfo]::new()
    $info.cbSize = [uint32]$infoSize
    $hr = [DwmTiming]::DwmGetCompositionTimingInfo($hwnd, [ref]$info)
    if ($hr -ne 0) { throw ('DwmGetCompositionTimingInfo failed: 0x{0:X8}; cbSize={1}; hwnd={2}' -f ($hr -band 0xffffffff), $infoSize, $hwnd) }
    if ($info.cFrame -ne $lastFrame -and $lastQpc -gt 0 -and $info.qpcCompose -gt $lastQpc) {
        $rows.Add([pscustomobject]@{
            elapsedMs = [math]::Round($clock.Elapsed.TotalMilliseconds, 2)
            frame = $info.cFrame
            qpc = $info.qpcCompose
            intervalMs = [math]::Round((($info.qpcCompose - $lastQpc) / $frequency) * 1000, 3)
            framesLate = $info.cFramesLate
            framesDisplayed = $info.cFramesDisplayed
            framesDropped = $info.cFramesDropped
            framesMissed = $info.cFramesMissed
            refreshRateHz = if ($info.rateRefresh.Denominator) { [math]::Round($info.rateRefresh.Numerator / $info.rateRefresh.Denominator, 2) } else { 0 }
        })
    }
    $lastFrame = $info.cFrame
    $lastQpc = $info.qpcCompose
    Start-Sleep -Milliseconds $SampleIntervalMs
}

$parent = Split-Path -Parent $OutputPath
if ($parent) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
$rows | Export-Csv -LiteralPath $OutputPath -NoTypeInformation
$intervals = @($rows | ForEach-Object { [double]$_.intervalMs } | Sort-Object)
$p = { param($fraction) if ($intervals.Count) { $intervals[[math]::Min($intervals.Count - 1, [math]::Floor($intervals.Count * $fraction))] } else { $null } }
$late = if ($rows.Count) { $rows[-1].framesLate } else { $null }
$missed = if ($rows.Count) { $rows[-1].framesMissed } else { $null }
$dropped = if ($rows.Count) { $rows[-1].framesDropped } else { $null }
"samples=$($rows.Count) p50Ms=$(& $p 0.50) p95Ms=$(& $p 0.95) p99Ms=$(& $p 0.99) maxMs=$(if($intervals.Count){$intervals[-1]}) late=$late missed=$missed dropped=$dropped qpcFrequency=$frequency scope=system-dwm-global output=$OutputPath"

