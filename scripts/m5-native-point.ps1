param([int]$RootPid)
$ErrorActionPreference='Stop'
Add-Type -TypeDefinition @'
using System; using System.Collections.Generic; using System.Runtime.InteropServices;
public class M5Processes {
 [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)] public struct Entry {
  public uint size,usage,pid; public UIntPtr heap; public uint module,threads,parent; public int priority; public uint flags;
  [MarshalAs(UnmanagedType.ByValTStr,SizeConst=260)] public string name;
 }
 [DllImport("kernel32.dll")] static extern IntPtr CreateToolhelp32Snapshot(uint flags,uint pid);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode)] static extern bool Process32FirstW(IntPtr handle,ref Entry entry);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode)] static extern bool Process32NextW(IntPtr handle,ref Entry entry);
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr handle);
 public static Dictionary<int,int> Parents() {
  var result=new Dictionary<int,int>(); var handle=CreateToolhelp32Snapshot(2,0);
  try { var entry=new Entry();entry.size=(uint)Marshal.SizeOf(entry);
   if(Process32FirstW(handle,ref entry)) do {result[(int)entry.pid]=(int)entry.parent;} while(Process32NextW(handle,ref entry));
  } finally {CloseHandle(handle);} return result;
 }
}
'@
$m5Parents=[M5Processes]::Parents();$m5Ids=@($RootPid)
do { $m5Children=@($m5Parents.Keys | Where-Object { $m5Parents[$_] -in $m5Ids -and $_ -notin $m5Ids });$m5Ids+=$m5Children } while($m5Children.Count)
$m5Rows=@(foreach($m5Id in $m5Ids) {
 $m5Process=Get-Process -Id $m5Id -ErrorAction SilentlyContinue
 if($m5Process) {try { [pscustomobject]@{pid=$m5Id;parent=$m5Parents[$m5Id];name=$m5Process.ProcessName;privateBytes=$m5Process.PrivateMemorySize64;handles=$m5Process.HandleCount;threads=$m5Process.Threads.Count} } finally {$m5Process.Dispose()} }
})
[pscustomobject]@{at=[DateTime]::UtcNow.ToString('o');processes=$m5Rows;processCount=$m5Rows.Count;privateBytes=($m5Rows|Measure-Object privateBytes -Sum).Sum;handles=($m5Rows|Measure-Object handles -Sum).Sum;threads=($m5Rows|Measure-Object threads -Sum).Sum} | ConvertTo-Json -Depth 5 -Compress
