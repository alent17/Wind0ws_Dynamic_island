param(
  [Parameter(Mandatory=$true)][string]$Publisher,
  [string]$CertificatePath,
  [string]$CertificatePassword,
  [string]$SdkBin
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
if (-not $SdkBin) {
  $kits = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
  $tool = Get-ChildItem -Path $kits -Filter makeappx.exe -Recurse -ErrorAction SilentlyContinue | Where-Object { $_.Directory.Name -eq 'x64' } | Sort-Object FullName -Descending | Select-Object -First 1
  if (-not $tool) { throw 'Windows SDK makeappx.exe was not found.' }
  $SdkBin = $tool.Directory.FullName
}
$version = (Get-Content -Raw (Join-Path $repo 'package.json') | ConvertFrom-Json).version + '.0'
if ($Publisher -notmatch '^CN=') { throw 'Publisher must be the certificate subject, beginning with CN=.' }
$publisherXml = [System.Security.SecurityElement]::Escape($Publisher)
Push-Location $repo
try {
  & npm run tauri -- build --no-bundle
  if ($LASTEXITCODE -ne 0) { throw 'Tauri build failed.' }
  $stage = Join-Path $repo 'src-tauri\target\msix-stage'
  New-Item -ItemType Directory -Force -Path (Join-Path $stage 'Assets') | Out-Null
  Copy-Item -LiteralPath (Join-Path $repo 'src-tauri\target\release\isle.exe') -Destination (Join-Path $stage 'isle.exe') -Force
  foreach ($icon in @('Square150x150Logo.png','Square44x44Logo.png','StoreLogo.png')) { Copy-Item -LiteralPath (Join-Path $repo "src-tauri\icons\$icon") -Destination (Join-Path $stage "Assets\$icon") -Force }
  $manifest = @"
<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10" xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10" xmlns:uap3="http://schemas.microsoft.com/appx/manifest/uap/windows10/3" xmlns:desktop="http://schemas.microsoft.com/appx/manifest/desktop/windows10" xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities" IgnorableNamespaces="uap uap3 desktop rescap">
  <Identity Name="Isle.DynamicIsland" Publisher="$publisherXml" Version="$version" ProcessorArchitecture="x64" />
  <Properties><DisplayName>Isle</DisplayName><PublisherDisplayName>Isle</PublisherDisplayName><Logo>Assets\StoreLogo.png</Logo></Properties>
  <Dependencies><TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.26100.0" /></Dependencies>
  <Resources><Resource Language="en-us" /></Resources>
  <Applications><Application Id="Isle" Executable="isle.exe" EntryPoint="Windows.FullTrustApplication"><uap:VisualElements DisplayName="Isle" Description="Isle Dynamic Island" BackgroundColor="transparent" Square150x150Logo="Assets\Square150x150Logo.png" Square44x44Logo="Assets\Square44x44Logo.png" /></Application></Applications>
  <Capabilities><uap3:Capability Name="userNotificationListener" /><rescap:Capability Name="runFullTrust" /></Capabilities>
</Package>
"@
  [System.IO.File]::WriteAllText((Join-Path $stage 'AppxManifest.xml'), $manifest, [System.Text.UTF8Encoding]::new($false))
  $output = Join-Path $repo "src-tauri\target\Isle_$($version)_x64.msix"
  & (Join-Path $SdkBin 'makeappx.exe') pack /d $stage /p $output /o
  if ($LASTEXITCODE -ne 0) { throw 'MSIX packaging failed.' }
  if ($CertificatePath) {
    $signArgs = @('sign','/fd','SHA256','/f',$CertificatePath)
    if ($CertificatePassword) { $signArgs += @('/p',$CertificatePassword) }
    & (Join-Path $SdkBin 'signtool.exe') @signArgs $output
    if ($LASTEXITCODE -ne 0) { throw 'MSIX signing failed.' }
  } else { Write-Warning 'Package is unsigned and must be signed with a trusted certificate before installation.' }
  Write-Host "Package: $output"
} finally { Pop-Location }
