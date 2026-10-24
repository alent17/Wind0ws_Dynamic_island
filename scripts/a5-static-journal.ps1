# Pure policy: broken evidence/stdout must never unwind past owned-input cleanup.
function Write-A10PacketSafely($Packet,[scriptblock]$Journal,[scriptblock]$Transport) {
    $report=@{serialized=$true;journalOk=$false;transportOk=$false;errors=@()}
    try{$line=$Packet | ConvertTo-Json -Compress -Depth 16}
    catch{$line='{"serializationFailed":true}';$report.serialized=$false;$report.errors+= $_.Exception.Message}
    try{$null=& $Journal $line;$report.journalOk=$true}catch{$report.errors+=$_.Exception.Message}
    try{$null=& $Transport $line;$report.transportOk=$true}catch{$report.errors+=$_.Exception.Message}
    return $report
}
function Invoke-A10RetainIfUnsafe($Release,$PhysicalIdle,[scriptblock]$Retain) {
    if(-not $Release.ok -or $PhysicalIdle -ne $true){$null=& $Retain;return $true}
    return $false
}
# The accepted owner/finally module is reused unchanged. Its repeated cleanup
# calls must not become further SendInput attempts after a known release fault.
function New-A10PressGate([scriptblock]$Backend) {
    return [pscustomobject]@{Backend=$Backend;ReleaseFailureObserved=$false}
}
function Invoke-A10GatedPress($Gate,[int]$Flags) {
    if($Gate.ReleaseFailureObserved){throw 'Prior release failed; native sends blocked, ownership retained.'}
    if($Flags -notin @(2,4)){throw 'Only separated left down/up permitted.'}
    try{$sent=& $Gate.Backend $Flags}
    catch{if($Flags -eq 4){$Gate.ReleaseFailureObserved=$true};throw}
    if($Flags -eq 4 -and $sent -ne 1){$Gate.ReleaseFailureObserved=$true}
    return $sent
}
