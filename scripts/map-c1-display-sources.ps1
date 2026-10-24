$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Sequential, Pack=4)] public struct DisplayLuid { public uint LowPart; public int HighPart; }
[StructLayout(LayoutKind.Sequential, Pack=4)] public struct DisplayRational { public uint Numerator; public uint Denominator; }
[StructLayout(LayoutKind.Sequential, Pack=4)] public struct DisplayPathSource { public DisplayLuid AdapterId; public uint Id; public uint ModeInfoIdx; public uint StatusFlags; }
[StructLayout(LayoutKind.Sequential, Pack=4)] public struct DisplayPathTarget { public DisplayLuid AdapterId; public uint Id; public uint ModeInfoIdx; public uint OutputTechnology; public uint Rotation; public uint Scaling; public DisplayRational RefreshRate; public uint ScanLineOrdering; [MarshalAs(UnmanagedType.Bool)] public bool TargetAvailable; public uint StatusFlags; }
[StructLayout(LayoutKind.Sequential, Pack=4)] public struct DisplayPathInfo { public DisplayPathSource Source; public DisplayPathTarget Target; public uint Flags; }
[StructLayout(LayoutKind.Explicit, Size=48)] public struct DisplayModeUnion { [FieldOffset(0)] public uint Width; [FieldOffset(4)] public uint Height; [FieldOffset(8)] public uint PixelFormat; [FieldOffset(12)] public int PositionX; [FieldOffset(16)] public int PositionY; }
[StructLayout(LayoutKind.Sequential, Pack=4)] public struct DisplayModeInfo { public uint InfoType; public uint Id; public DisplayLuid AdapterId; public DisplayModeUnion Data; }
[StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode, Pack=4)] public struct DisplayConfigHeader { public uint Type; public uint Size; public DisplayLuid AdapterId; public uint Id; }
[StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode, Pack=4)] public struct DisplaySourceName { public DisplayConfigHeader Header; [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string ViewGdiDeviceName; }
[StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode, Pack=4)] public struct DisplayTargetName { public DisplayConfigHeader Header; public uint Flags; public uint OutputTechnology; public ushort EdidManufactureId; public ushort EdidProductCodeId; public uint ConnectorInstance; [MarshalAs(UnmanagedType.ByValTStr, SizeConst=64)] public string MonitorFriendlyDeviceName; [MarshalAs(UnmanagedType.ByValTStr, SizeConst=128)] public string MonitorDevicePath; }

public static class DisplayConfigApi {
  [DllImport("user32.dll")] public static extern int GetDisplayConfigBufferSizes(uint flags, out uint pathCount, out uint modeCount);
  [DllImport("user32.dll")] public static extern int QueryDisplayConfig(uint flags, ref uint pathCount, [Out] DisplayPathInfo[] paths, ref uint modeCount, [Out] DisplayModeInfo[] modes, IntPtr topologyId);
  [DllImport("user32.dll", EntryPoint="DisplayConfigGetDeviceInfo")] public static extern int GetSourceName(ref DisplaySourceName name);
  [DllImport("user32.dll", EntryPoint="DisplayConfigGetDeviceInfo")] public static extern int GetTargetName(ref DisplayTargetName name);
}
'@

$pathCount = [uint32]0; $modeCount = [uint32]0
$qdcOnlyActivePaths = [uint32]2
$result = [DisplayConfigApi]::GetDisplayConfigBufferSizes($qdcOnlyActivePaths, [ref]$pathCount, [ref]$modeCount)
if ($result -ne 0) { throw "GetDisplayConfigBufferSizes failed: $result" }
$paths = [DisplayPathInfo[]]::new($pathCount)
$modes = [DisplayModeInfo[]]::new($modeCount)
$result = [DisplayConfigApi]::QueryDisplayConfig($qdcOnlyActivePaths, [ref]$pathCount, $paths, [ref]$modeCount, $modes, [IntPtr]::Zero)
if ($result -ne 0) { throw "QueryDisplayConfig failed: $result" }
for ($i=0; $i -lt $pathCount; $i++) {
  $path = $paths[$i]
  $source = [DisplaySourceName]::new(); $source.Header.Type = 1; $source.Header.Size = [Runtime.InteropServices.Marshal]::SizeOf([type][DisplaySourceName]); $source.Header.AdapterId = $path.Source.AdapterId; $source.Header.Id = $path.Source.Id
  $target = [DisplayTargetName]::new(); $target.Header.Type = 2; $target.Header.Size = [Runtime.InteropServices.Marshal]::SizeOf([type][DisplayTargetName]); $target.Header.AdapterId = $path.Target.AdapterId; $target.Header.Id = $path.Target.Id
  $sourceResult = [DisplayConfigApi]::GetSourceName([ref]$source)
  $targetResult = [DisplayConfigApi]::GetTargetName([ref]$target)
  $sourceMode = $null
  if ($path.Source.ModeInfoIdx -ne [uint32]::MaxValue -and $path.Source.ModeInfoIdx -lt $modeCount -and $modes[$path.Source.ModeInfoIdx].InfoType -eq 1) { $sourceMode = $modes[$path.Source.ModeInfoIdx].Data }
  [pscustomobject]@{ VidPnSourceId = $path.Source.Id; SourceModeIndex = $path.Source.ModeInfoIdx; SourceModeX = $sourceMode?.PositionX; SourceModeY = $sourceMode?.PositionY; SourceModeWidth = $sourceMode?.Width; SourceModeHeight = $sourceMode?.Height; GdiDeviceName = $source.ViewGdiDeviceName; TargetId = $path.Target.Id; MonitorName = $target.MonitorFriendlyDeviceName; MonitorDevicePath = $target.MonitorDevicePath; RefreshHz = if ($path.Target.RefreshRate.Denominator) { [math]::Round($path.Target.RefreshRate.Numerator / $path.Target.RefreshRate.Denominator, 3) } else { $null }; SourceQueryResult = $sourceResult; TargetQueryResult = $targetResult }
}
