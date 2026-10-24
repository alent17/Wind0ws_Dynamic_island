param(
    [Parameter(Mandatory = $true)][int]$ProcessId,
    [Parameter(Mandatory = $true)][string]$OutputCsv,
    [int]$Seconds = 120,
    [int]$SampleIntervalMs = 8
)
$ErrorActionPreference = 'Stop'

Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Runtime.InteropServices;

public static class WindowProbe {
  public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);
  [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] public struct MonitorInfo {
    public int Size; public Rect Monitor; public Rect Work; public uint Flags;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string Device;
  }
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out Rect rect);
  [DllImport("user32.dll")] public static extern IntPtr MonitorFromWindow(IntPtr hwnd, uint flags);
  [DllImport("user32.dll", EntryPoint="GetMonitorInfoW", SetLastError=true, CharSet=CharSet.Unicode)] public static extern bool GetMonitorInfoW(IntPtr monitor, ref MonitorInfo info);
  public static IntPtr FindTopWindow(int pid) {
    IntPtr found = IntPtr.Zero;
    EnumWindows((hwnd, _) => { uint owner; GetWindowThreadProcessId(hwnd, out owner); if (owner == pid && IsWindowVisible(hwnd)) { found = hwnd; return false; } return true; }, IntPtr.Zero);
    return found;
  }
  public static string DeviceForWindow(IntPtr hwnd) {
    var monitor = MonitorFromWindow(hwnd, 2); var info = new MonitorInfo(); info.Size = Marshal.SizeOf(typeof(MonitorInfo));
    return GetMonitorInfoW(monitor, ref info) ? info.Device : "unknown(error=" + Marshal.GetLastWin32Error() + ",size=" + info.Size + ",handle=" + monitor.ToInt64() + ")";
  }
}
'@

$resolved = [System.IO.Path]::GetFullPath($OutputCsv)
New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetDirectoryName($resolved)) | Out-Null
$rows = [System.Collections.Generic.List[object]]::new()
$deadline = [DateTime]::UtcNow.AddSeconds($Seconds)
$hwnd = [WindowProbe]::FindTopWindow($ProcessId)
if ($hwnd -eq [IntPtr]::Zero) { throw "No visible top-level HWND found for PID $ProcessId" }
$frequency = [System.Diagnostics.Stopwatch]::Frequency
while ([DateTime]::UtcNow -lt $deadline) {
  $rect = [WindowProbe+Rect]::new()
  [void][WindowProbe]::GetWindowRect($hwnd, [ref]$rect)
  $rows.Add([pscustomobject]@{
    Utc = [DateTime]::UtcNow.ToString('o'); Qpc = [System.Diagnostics.Stopwatch]::GetTimestamp(); QpcFrequency = $frequency
    ProcessId = $ProcessId; Hwnd = ('0x{0:X}' -f $hwnd.ToInt64()); MonitorDevice = [WindowProbe]::DeviceForWindow($hwnd)
    Left = $rect.Left; Top = $rect.Top; Right = $rect.Right; Bottom = $rect.Bottom
    Width = $rect.Right - $rect.Left; Height = $rect.Bottom - $rect.Top
  })
  Start-Sleep -Milliseconds $SampleIntervalMs
}
$rows | Export-Csv -LiteralPath $resolved -NoTypeInformation -Encoding utf8
