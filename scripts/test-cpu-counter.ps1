$ErrorActionPreference='Stop'
$smallDelta=[double]0.015625
$kept=[math]::Max([double]0,$smallDelta)
if($kept -ne $smallDelta) { throw 'CPU delta was rounded to zero' }
$normalized=100*$kept/[double]1.5/[double]12
if([math]::Abs($normalized-0.0868055555555556) -gt 1e-12) { throw 'CPU normalization failed' }
if([math]::Max([double]0,[double]-0.015625) -ne 0) { throw 'Counter reset was not clamped' }
Write-Output 'Fractional CPU delta, machine normalization, and reset clamp passed.'
