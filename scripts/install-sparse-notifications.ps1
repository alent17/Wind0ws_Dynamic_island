param(
  [string]$SdkBin,
  [switch]$Uninstall
)
$ErrorActionPreference = 'Stop'
$identityName = 'Isle.DynamicIsland'
$publisher = 'CN=Isle Local Notifications'
$repo = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $env:LOCALAPPDATA 'Isle\AppxManifest.xml'
$certPath = Join-Path $env:LOCALAPPDATA 'Isle\Isle-Notifications.cer'

if ($Uninstall) {
  Get-AppxPackage -Name $identityName | Remove-AppxPackage
  exit 0
}

if (-not $SdkBin) {
  $kits = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
  $makeappx = Get-ChildItem -Path $kits -Filter makeappx.exe -Recurse | Where-Object { $_.Directory.Name -eq 'x64' } | Sort-Object FullName -Descending | Select-Object -First 1
  $SdkBin = $makeappx.Directory.FullName
}
$makeappxPath = Join-Path $SdkBin 'makeappx.exe'
$signtoolPath = Join-Path $SdkBin 'signtool.exe'
$app = Join-Path $repo 'src-tauri\target\release\isle.exe'
$stage = Join-Path $env:LOCALAPPDATA 'Isle\sparse-package'
New-Item -ItemType Directory -Force -Path $stage | Out-Null
New-Item -ItemType Directory -Force -Path (Split-Path $manifestPath) | Out-Null
$version = (Get-Content -Raw (Join-Path $repo 'package.json') | ConvertFrom-Json).version + '.0'
$escapedPublisher = [System.Security.SecurityElement]::Escape($publisher)
$escapedPath = [System.Security.SecurityElement]::Escape($app)
$manifest = @"
<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10" xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10" xmlns:uap3="http://schemas.microsoft.com/appx/manifest/uap/windows10/3" xmlns:desktop="http://schemas.microsoft.com/appx/manifest/desktop/windows10" xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities" IgnorableNamespaces="uap uap3 desktop rescap">
  <Identity Name="$identityName" Publisher="$escapedPublisher" Version="$version" ProcessorArchitecture="x64" />
  <Properties><DisplayName>Isle</DisplayName><PublisherDisplayName>Isle Local</PublisherDisplayName><Logo>$(Join-Path $repo 'src-tauri\icons\StoreLogo.png')</Logo></Properties>
  <Dependencies><TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.26100.0" /></Dependencies>
  <Applications><Application Id="Isle" Executable="$escapedPath" EntryPoint="Windows.FullTrustApplication"><uap:VisualElements DisplayName="Isle" Description="Isle Dynamic Island" BackgroundColor="transparent" Square150x150Logo="$(Join-Path $repo 'src-tauri\icons\Square150x150Logo.png')" Square44x44Logo="$(Join-Path $repo 'src-tauri\icons\Square44x44Logo.png')" /></Application></Applications>
  <Capabilities><uap3:Capability Name="userNotificationListener" /><rescap:Capability Name="runFullTrust" /></Capabilities>
</Package>
"@
[System.IO.File]::WriteAllText($manifestPath, $manifest, [System.Text.UTF8Encoding]::new($false))
Copy-Item -LiteralPath $manifestPath -Destination (Join-Path $stage 'AppxManifest.xml') -Force
if (-not (Test-Path $certPath)) {
  $cert = New-SelfSignedCertificate -Type Custom -Subject $publisher -KeyUsage DigitalSignature -FriendlyName 'Isle local notification identity' -CertStoreLocation 'Cert:\CurrentUser\My' -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3')
  Export-Certificate -Cert $cert -FilePath $certPath | Out-Null
  Import-Certificate -FilePath $certPath -CertStoreLocation 'Cert:\CurrentUser\TrustedPeople' | Out-Null
}
$signedManifest = Join-Path $stage 'AppxManifest.xml'
& $signtoolPath sign /fd SHA256 /f $certPath $signedManifest
if ($LASTEXITCODE -ne 0) { throw 'Sparse manifest signing failed.' }
& $makeappxPath pack /d $stage /p (Join-Path $stage 'Isle-Notifications.msix') /o
if ($LASTEXITCODE -ne 0) { throw 'Sparse package creation failed.' }
Add-AppxPackage -Path (Join-Path $stage 'Isle-Notifications.msix') -ExternalLocation $app
Write-Host 'Sparse package registered. Start Isle and enable Windows notifications in Settings.'
