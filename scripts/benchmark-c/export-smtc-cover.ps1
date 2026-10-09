param([string]$ExpectedTitle,[string]$OutputFile)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$adapter=[System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { $_.Name -eq 'AsTask' -and $_.IsGenericMethod -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' } | Select-Object -First 1
function Await-Operation($operation,$type){$task=$adapter.MakeGenericMethod($type).Invoke($null,@($operation));$task.GetAwaiter().GetResult()}
$manager=Await-Operation ([Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager,Windows.Media.Control,ContentType=WindowsRuntime]::RequestAsync()) ([Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager,Windows.Media.Control,ContentType=WindowsRuntime])
for($attempt=0;$attempt -lt 30;$attempt++){
    $session=$manager.GetSessions() | Where-Object {$_.SourceAppUserModelId -like '*cloudmusic.exe*'} | Select-Object -First 1
    $props=Await-Operation ($session.TryGetMediaPropertiesAsync()) ([Windows.Media.Control.GlobalSystemMediaTransportControlsSessionMediaProperties,Windows.Media.Control,ContentType=WindowsRuntime])
    if($props.Title -eq $ExpectedTitle -and $props.Thumbnail){break}
    Start-Sleep -Milliseconds 200
}
if($props.Title -ne $ExpectedTitle -or -not $props.Thumbnail){throw 'Requested real track thumbnail not available'}
$stream=Await-Operation ($props.Thumbnail.OpenReadAsync()) ([Windows.Storage.Streams.IRandomAccessStreamWithContentType,Windows.Storage.Streams,ContentType=WindowsRuntime])
$randomType=[Windows.Storage.Streams.IRandomAccessStream,Windows.Storage.Streams,ContentType=WindowsRuntime]
$size=$randomType.GetProperty('Size').GetValue($stream)
if($size -eq 0 -or $size -gt 12582912){throw 'Thumbnail outside export budget'}
$input=$randomType.GetMethod('GetInputStreamAt').Invoke($stream,@([uint64]0))
$reader=[Windows.Storage.Streams.DataReader,Windows.Storage.Streams,ContentType=WindowsRuntime]::new($input)
$null=Await-Operation ($reader.LoadAsync([uint32]$size)) ([uint32])
$bytes=New-Object byte[] ([int]$size);$reader.ReadBytes($bytes)
[IO.File]::WriteAllBytes([IO.Path]::GetFullPath($OutputFile),$bytes)
Write-Output "Exported real SMTC thumbnail for $($props.Title): $($bytes.Length) bytes"
