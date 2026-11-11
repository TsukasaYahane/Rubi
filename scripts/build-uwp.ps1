# Builds the UWP app with the pinned nightly and lays it out as a loose package. With -Register it
# also registers the layout on this PC (needs Developer Mode; no certificate is involved). With
# -Package it packs a separate layout into dist\rubi-uwp-x64.msix, signed with the self-signed CN=Rubi
# certificate from the current user's store (made on first use), and leaves its public half beside
# it as dist\rubi-uwp.cer. With -Store it packs dist\rubi-store-x64.msix for Partner Center instead:
# unsigned (the Store signs it) and carrying the identity the Store reserved for the app, which only
# that package has: the registered copy keeps its own, so its data stays. It leaves a registered or
# running copy alone and trusts nothing.
param(
    [ValidateSet('debug', 'release')] [string] $Profile = 'debug',
    [switch] $Register,
    [switch] $Package,
    [switch] $Store
)
$ErrorActionPreference = 'Stop'
if ($Package -and $Store) { throw '-Package and -Store each pack their own package; pick one.' }

# Partner Center: Product management > Product identity.
$storeName = '75B213EC.MetroBi'
$storePublisher = 'CN=8DE47572-E1DD-4D84-8C1F-3444FEB72C17'
# The Store takes only a name reserved for the app.
$storeDisplayName = 'MetroBi'

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

$layout = Join-Path $env:CARGO_TARGET_DIR $(if ($Store) { 'uwp-store' } elseif ($Package) { 'uwp-package' } else { 'uwp-layout' })
if (($Package -or $Store) -and (Test-Path $layout)) { Remove-Item $layout -Recurse -Force }
New-Item -ItemType Directory -Force $layout | Out-Null
Copy-Item (Join-Path $env:CARGO_TARGET_DIR "$target\$Profile\rubi.exe") $layout -Force
Copy-Item (Join-Path $root 'windows\uwp\AppxManifest.xml') $layout -Force
Copy-Item (Join-Path $root 'windows\uwp\Assets') $layout -Recurse -Force
if ($Store) {
    $manifestPath = Join-Path $layout 'AppxManifest.xml'
    $manifest = New-Object System.Xml.XmlDocument
    $manifest.PreserveWhitespace = $true
    $manifest.Load($manifestPath)
    $manifest.Package.Identity.SetAttribute('Name', $storeName)
    $manifest.Package.Identity.SetAttribute('Publisher', $storePublisher)
    $manifest.Package.Properties.DisplayName = $storeDisplayName
    $visual = $manifest.Package.Applications.Application.VisualElements
    $visual.SetAttribute('DisplayName', $storeDisplayName)
    $visual.DefaultTile.SetAttribute('ShortName', $storeDisplayName)
    $manifest.Save($manifestPath)
}
Write-Host "Layout: $layout"

if ($Register) {
    Add-AppxPackage -Register (Join-Path $layout 'AppxManifest.xml')
    $registered = Get-AppxPackage -Name 'Rubi'
    Write-Host "Registered: $($registered.PackageFamilyName)"
}

if ($Package) {
    $tools = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.*\x64\makeappx.exe" |
        Sort-Object FullName | Select-Object -Last 1 | Split-Path -Parent
    $subject = 'CN=Rubi'
    $certificate = Get-ChildItem Cert:\CurrentUser\My |
        Where-Object { $_.Subject -eq $subject -and $_.HasPrivateKey -and $_.NotAfter -gt (Get-Date) } |
        Sort-Object NotAfter | Select-Object -Last 1
    if (-not $certificate) {
        $certificate = New-SelfSignedCertificate -Type Custom -Subject $subject -KeyUsage DigitalSignature `
            -FriendlyName 'Rubi package signing' -CertStoreLocation Cert:\CurrentUser\My `
            -NotAfter (Get-Date).AddYears(5) `
            -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')
    }
    $dist = Join-Path $root 'dist'
    New-Item -ItemType Directory -Force $dist | Out-Null
    $msix = Join-Path $dist 'rubi-uwp-x64.msix'
    & (Join-Path $tools 'makeappx.exe') pack /o /d $layout /p $msix
    if ($LASTEXITCODE -ne 0) { throw "makeappx failed with $LASTEXITCODE" }
    & (Join-Path $tools 'signtool.exe') sign /fd SHA256 /sha1 $certificate.Thumbprint /s My $msix
    if ($LASTEXITCODE -ne 0) { throw "signtool failed with $LASTEXITCODE" }
    Export-Certificate -Cert $certificate -FilePath (Join-Path $dist 'rubi-uwp.cer') | Out-Null
    Write-Host "Package: $msix ($([math]::Round((Get-Item $msix).Length / 1MB, 2)) MB)"
}

if ($Store) {
    $tools = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.*\x64\makeappx.exe" |
        Sort-Object FullName | Select-Object -Last 1 | Split-Path -Parent
    $dist = Join-Path $root 'dist'
    New-Item -ItemType Directory -Force $dist | Out-Null
    $msix = Join-Path $dist 'rubi-store-x64.msix'
    & (Join-Path $tools 'makeappx.exe') pack /o /d $layout /p $msix
    if ($LASTEXITCODE -ne 0) { throw "makeappx failed with $LASTEXITCODE" }
    Write-Host "Store package: $msix ($([math]::Round((Get-Item $msix).Length / 1MB, 2)) MB)"
}
