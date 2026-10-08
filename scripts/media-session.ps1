param([ValidateSet('Snapshot','Pause','Play','Next')][string]$Action='Snapshot', [string]$Source='cloudmusic.exe')
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$managerType=[Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager,Windows.Media.Control,ContentType=WindowsRuntime]
$boolType=[bool]
$propsType=[Windows.Media.Control.GlobalSystemMediaTransportControlsSessionMediaProperties,Windows.Media.Control,ContentType=WindowsRuntime]
$adapter=[System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { $_.Name -eq 'AsTask' -and $_.IsGenericMethod -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' } | Select-Object -First 1
function Await-Operation($operation,$type) {
    $task=$adapter.MakeGenericMethod($type).Invoke($null,@($operation))
    $task.GetAwaiter().GetResult()
}
$manager=Await-Operation ($managerType::RequestAsync()) $managerType
$rows=@(foreach($session in $manager.GetSessions()) {
    $id=$session.SourceAppUserModelId
    if($Action -ne 'Snapshot' -and $id -like "*$Source*") {
        $operation=switch($Action) { 'Pause' {$session.TryPauseAsync()} 'Play' {$session.TryPlayAsync()} 'Next' {$session.TrySkipNextAsync()} }
        if(-not (Await-Operation $operation $boolType)) { throw "Player rejected $Action" }
    }
    $props=Await-Operation ($session.TryGetMediaPropertiesAsync()) $propsType
    [pscustomobject]@{ source=$id; title=$props.Title; artist=$props.Artist; playbackStatus=$session.GetPlaybackInfo().PlaybackStatus.ToString(); hasThumbnail=($null -ne $props.Thumbnail) }
})
ConvertTo-Json -InputObject $rows -Depth 4
