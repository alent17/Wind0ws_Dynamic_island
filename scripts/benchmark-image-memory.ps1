param([string]$OutputDirectory = 'docs/performance/round2-2026-10-24')
$ErrorActionPreference = 'Stop'
$workspace = Split-Path -Parent $PSScriptRoot
$dependencies = Join-Path $workspace 'src-tauri/target/release/deps'
$output = [IO.Path]::GetFullPath((Join-Path $workspace $OutputDirectory))
New-Item -ItemType Directory -Force -Path $output | Out-Null
$scratch = Join-Path $workspace 'dist/performance/round2-image-probe'
New-Item -ItemType Directory -Force -Path $scratch | Out-Null
$executable = Join-Path $scratch 'image-memory-probe.exe'
$imageLibrary = Get-ChildItem -LiteralPath $dependencies -Filter 'libimage-*.rlib' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$base64Library = Get-ChildItem -LiteralPath $dependencies -Filter 'libbase64-*.rlib' | Where-Object {
    $dependencyFile = Join-Path $dependencies ($_.BaseName.Substring(3) + '.d')
    (Test-Path -LiteralPath $dependencyFile) -and ((Get-Content -LiteralPath $dependencyFile -Raw) -match 'base64-0\.21\.')
} | Select-Object -First 1
if (-not $imageLibrary -or -not $base64Library) { throw 'Build Release dependencies first.' }
& rustc --edition=2021 -O -C panic=abort -A dead_code (Join-Path $PSScriptRoot 'benchmark-image-memory.rs') -L "dependency=$dependencies" --extern "image=$($imageLibrary.FullName)" --extern "base64=$($base64Library.FullName)" -o $executable
if ($LASTEXITCODE -ne 0) { throw 'Probe compilation failed.' }
$fixture = Join-Path $scratch 'rgba-1280.png'
& $executable fixture $fixture
if ($LASTEXITCODE -ne 0) { throw 'Fixture generation failed.' }
$rows = @()
for ($round = 1; $round -le 3; $round++) {
    # Alternate ordering to reduce systematic warm-cache bias.
    $modes = if ($round % 2) { @('legacy-normal','optimized-normal','legacy-pixel','optimized-pixel') } else { @('optimized-pixel','legacy-pixel','optimized-normal','legacy-normal') }
    foreach ($mode in $modes) {
        $row = (& $executable $mode $fixture) | ConvertFrom-Json
        if ($LASTEXITCODE -ne 0) { throw "Probe failed: $mode" }
        $row | Add-Member -NotePropertyName round -NotePropertyValue $round
        $rows += $row
    }
}
foreach ($kind in @('normal','pixel')) {
    $hashes = @($rows | Where-Object { $_.mode.EndsWith($kind) } | Select-Object -ExpandProperty outputHash -Unique)
    if ($hashes.Count -ne 1) { throw "Output mismatch for $kind" }
}
$report = [ordered]@{
    createdUtc = [DateTime]::UtcNow.ToString('o')
    scope = 'Isolated Rust requested heap allocations; production optimized image/budget/utils modules, prior algorithm as reference; no WebView, Windows working set or GPU. Same deterministic 1280x1280 RGBA PNG; three fresh-process repetitions.'
    sourceHashes = @(foreach ($path in @('src-tauri/src/services/image.rs','src-tauri/src/services/image_budget.rs','src-tauri/src/utils.rs','src-tauri/Cargo.lock','scripts/benchmark-image-memory.rs')) { @{path=$path;sha256=(Get-FileHash -LiteralPath (Join-Path $workspace $path)).Hash} })
    fixtureSha256 = (Get-FileHash -LiteralPath $fixture).Hash
    rows = $rows
}
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $output 'image-allocation.json') -Encoding utf8
$rows | Format-Table mode,round,peakRequestedBytes,totalAllocatedBytes,elapsedMs,outputBytes
