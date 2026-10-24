$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'a5-static-journal.ps1')
. (Join-Path $PSScriptRoot 'a5-owned-press.ps1')
$script:retained=$false
$delivery=Write-A10PacketSafely @{ok=$true} {throw 'fake disk failure'} {throw 'fake closed pipe'}
if($delivery.journalOk -or $delivery.transportOk -or $delivery.errors.Count -ne 2){throw 'Delivery failure hidden'}
$trace=[Collections.Generic.List[string]]::new()
$owner=New-A5PressOwner {param($flags) $trace.Add("inject:$flags");if($flags -eq 4){return 0};return 1}
Invoke-A5PressSession $owner { $null=Invoke-A5OwnedDown $owner } {param($release)
    $null=Write-A10PacketSafely @{release=$release} {throw 'fake disk failure'} {throw 'fake EOF pipe'}
    $null=Invoke-A10RetainIfUnsafe $release $false { $script:retained=$true }
}
if(-not $script:retained -or -not $owner.Held -or ($trace -join ',') -ne 'inject:2,inject:4'){throw 'Broken output bypassed release/retention'}
$script:retained=$false
$normal=Invoke-A10RetainIfUnsafe @{ok=$true} $true { $script:retained=$true }
if($normal -or $script:retained){throw 'Safe cleanup incorrectly retained'}
$blocked=Invoke-A10RetainIfUnsafe @{ok=$true} $null { $script:retained=$true }
if(-not $blocked -or -not $script:retained){throw 'Unknown physical input treated as idle'}
foreach($failureKind in @('zero','throw')) {
    $gateTrace=[Collections.Generic.List[string]]::new()
    $gate=New-A10PressGate {param($flags) $gateTrace.Add("inject:$flags");if($flags -eq 4){if($failureKind -eq 'throw'){throw 'fake native release exception'};return 0};return 1}
    $gatedOwner=New-A5PressOwner {param($flags) Invoke-A10GatedPress $gate $flags}
    $script:gatedRetained=$false;$script:restoredAfterFault=$false
    Invoke-A5PressSession $gatedOwner {
        $null=Invoke-A5OwnedDown $gatedOwner
        try{$null=Invoke-A5OwnedUp $gatedOwner}catch{$null=Close-A5OwnedPress $gatedOwner}
    } {param($release)
        $null=Invoke-A10RetainIfUnsafe $release $false {$script:gatedRetained=$true}
        $null=Invoke-A5RestoreWhenReleased $release {$script:restoredAfterFault=$true}
    }
    if(($gateTrace -join ',') -ne 'inject:2,inject:4' -or -not $gate.ReleaseFailureObserved -or -not $gatedOwner.Held -or -not $script:gatedRetained -or $script:restoredAfterFault){throw "Error/finally repeated native release after $failureKind failure"}
}
@{passed=5;guiCreated=$false;nativeInjected=$false;cases=@('disk+stdout failures do not throw','EOF cleanup release failure retains instead of moving/disposal','unknown physical input blocks restore','normal up returns zero: catch+EOF/finally cannot repeat native release','normal up throws: catch+EOF/finally cannot repeat native release')} | ConvertTo-Json -Depth 4
