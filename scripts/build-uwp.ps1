# Builds the UWP app with the pinned nightly and lays it out as a loose package. With -Register it
# also registers the layout on this PC (needs Developer Mode; no certificate is involved).
param(
    [ValidateSet('debug', 'release')] [string] $Profile = 'debug',
    [switch] $Register
)
$ErrorActionPreference = 'Stop'

$root = Split-Path $PSScriptRoot -Parent
$toolchain = 'nightly-2026-10-15'
$target = 'x86_64-uwp-windows-msvc'
if (-not $env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR = 'D:\rubi-uwp-target' }
if (-not $env:CARGO_HOME) { $env:CARGO_HOME = 'D:\rubi-cache\cargo' }

$cargoArgs = @("+$toolchain", 'build', '-p', 'rubi-uwp', '--target', $target, '-Z', 'build-std=std,panic_abort')
if ($Profile -eq 'release') { $cargoArgs += '--release' }
Push-Location $root
try {
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) { throw "cargo failed with $LASTEXITCODE" }
} finally {
    Pop-Location
}

$layout = Join-Path $env:CARGO_TARGET_DIR 'uwp-layout'
New-Item -ItemType Directory -Force $layout | Out-Null
Copy-Item (Join-Path $env:CARGO_TARGET_DIR "$target\$Profile\rubi.exe") $layout -Force
Copy-Item (Join-Path $root 'windows\uwp\AppxManifest.xml') $layout -Force
Copy-Item (Join-Path $root 'windows\uwp\Assets') $layout -Recurse -Force
Write-Host "Layout: $layout"

if ($Register) {
    Add-AppxPackage -Register (Join-Path $layout 'AppxManifest.xml')
    $package = Get-AppxPackage -Name 'Rubi'
    Write-Host "Registered: $($package.PackageFamilyName)"
}
