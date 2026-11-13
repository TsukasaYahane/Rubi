# Builds the UWP app with the pinned nightly for each of -Arch (x64 unless told; arm64 for Surface on
# ARM, which needs LLVM's clang) and lays each out as a loose package.
#
# With -Register it also registers the x64 layout on this PC (needs Developer Mode; no certificate is
# involved). With -Package it packs the layouts for GitHub, signed with the self-signed CN=Rubi
# certificate from the current user's store (made on first use), and leaves its public half beside it
# as dist\rubi-uwp.cer. With -Store it packs them for Partner Center instead: unsigned (the Store signs
# them) and carrying the identity the Store reserved for the app, which only those packages have: the
# registered copy keeps its own, so its data stays. One architecture packs to
# dist\rubi-{uwp,store}-<arch>.msix, several to one dist\rubi-{uwp,store}.msixbundle.
#
# It leaves a registered or running copy alone and trusts nothing.
param(
    [ValidateSet('debug', 'release')] [string] $Profile = 'debug',
    # x64, arm64, or both as x64,arm64 (which -File hands over as one string).
    [string[]] $Arch = @('x64'),
    [switch] $Register,
    [switch] $Package,
    [switch] $Store
)
$ErrorActionPreference = 'Stop'
if ($Package -and $Store) { throw '-Package and -Store each pack their own package; pick one.' }
$Arch = @($Arch -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ } | Select-Object -Unique)
foreach ($cpu in $Arch) { if ($cpu -notin 'x64', 'arm64') { throw "Unknown -Arch '$cpu'; use x64 or arm64." } }
if ($Register -and $Arch -notcontains 'x64') { throw '-Register registers the x64 layout; add x64 to -Arch.' }

# Partner Center: Product management > Product identity.
$storeName = '75B213EC.MetroBi'
$storePublisher = 'CN=8DE47572-E1DD-4D84-8C1F-3444FEB72C17'
# The Store takes only a name reserved for the app.
$storeDisplayName = 'MetroBi'

$root = Split-Path $PSScriptRoot -Parent
$toolchain = 'nightly-2026-10-15'
$targets = @{ x64 = 'x86_64-uwp-windows-msvc'; arm64 = 'aarch64-uwp-windows-msvc' }
if (-not $env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR = 'D:\rubi-uwp-target' }
if (-not $env:CARGO_HOME) { $env:CARGO_HOME = 'D:\rubi-cache\cargo' }
# The C in the dependencies builds for ARM with clang only.
$llvm = Join-Path $env:ProgramFiles 'LLVM\bin'
if ($Arch -contains 'arm64' -and -not (Get-Command clang -ErrorAction SilentlyContinue) -and (Test-Path $llvm)) {
    $env:PATH = "$llvm;$env:PATH"
}

$kind = if ($Store) { 'store' } elseif ($Package) { 'package' } else { 'layout' }
$layouts = @{}
foreach ($cpu in $Arch) {
    $target = $targets[$cpu]
    $cargoArgs = @("+$toolchain", 'build', '-p', 'rubi-uwp', '--target', $target, '-Z', 'build-std=std,panic_abort')
    if ($Profile -eq 'release') { $cargoArgs += '--release' }
    Push-Location $root
    try {
        & cargo @cargoArgs
        if ($LASTEXITCODE -ne 0) { throw "cargo failed with $LASTEXITCODE" }
    } finally {
        Pop-Location
    }

    # The x64 layout keeps its old place, where this PC's registration points.
    $name = if ($kind -eq 'layout' -and $cpu -eq 'x64') { 'uwp-layout' } else { "uwp-$kind-$cpu" }
    $layout = Join-Path $env:CARGO_TARGET_DIR $name
    if ($kind -ne 'layout' -and (Test-Path $layout)) { Remove-Item $layout -Recurse -Force }
    New-Item -ItemType Directory -Force $layout | Out-Null
    Copy-Item (Join-Path $env:CARGO_TARGET_DIR "$target\$Profile\rubi.exe") $layout -Force
    Copy-Item (Join-Path $root 'windows\uwp\AppxManifest.xml') $layout -Force
    Copy-Item (Join-Path $root 'windows\uwp\Assets') $layout -Recurse -Force
    $manifestPath = Join-Path $layout 'AppxManifest.xml'
    $manifest = New-Object System.Xml.XmlDocument
    $manifest.PreserveWhitespace = $true
    $manifest.Load($manifestPath)
    $manifest.Package.Identity.SetAttribute('ProcessorArchitecture', $cpu)
    if ($Store) {
        $manifest.Package.Identity.SetAttribute('Name', $storeName)
        $manifest.Package.Identity.SetAttribute('Publisher', $storePublisher)
        $manifest.Package.Properties.DisplayName = $storeDisplayName
        $visual = $manifest.Package.Applications.Application.VisualElements
        $visual.SetAttribute('DisplayName', $storeDisplayName)
        $visual.DefaultTile.SetAttribute('ShortName', $storeDisplayName)
    }
    $manifest.Save($manifestPath)
    $layouts[$cpu] = $layout
    Write-Host "Layout ($cpu): $layout"
}

if ($Register) {
    # Windows keeps an already registered copy where it is, even when the same version is registered
    # again from another folder: the exe there would stay the old one. So the fresh layout is copied
    # to the registered folder, which also keeps that copy's local data and its sign-in.
    $layout = $layouts['x64']
    $existing = Get-AppxPackage -Name 'Rubi' -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($existing -and $existing.InstallLocation -and
        ($existing.InstallLocation.TrimEnd('\') -ne $layout.TrimEnd('\'))) {
        Copy-Item (Join-Path $layout '*') $existing.InstallLocation -Recurse -Force
        $registered = $existing
    } else {
        Add-AppxPackage -Register (Join-Path $layout 'AppxManifest.xml')
        $registered = Get-AppxPackage -Name 'Rubi'
    }
    Write-Host "Registered: $($registered.PackageFamilyName) at $($registered.InstallLocation)"
}

if (-not ($Package -or $Store)) { return }

$tools = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.*\x64\makeappx.exe" |
    Sort-Object FullName | Select-Object -Last 1 | Split-Path -Parent
$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force $dist | Out-Null
$stem = if ($Store) { 'rubi-store' } else { 'rubi-uwp' }

if ($Arch.Count -eq 1) {
    $output = Join-Path $dist "$stem-$($Arch[0]).msix"
    & (Join-Path $tools 'makeappx.exe') pack /o /d $layouts[$Arch[0]] /p $output
    if ($LASTEXITCODE -ne 0) { throw "makeappx failed with $LASTEXITCODE" }
} else {
    # Each architecture's package goes into a folder of its own, which the bundle is made from.
    $parts = Join-Path $env:CARGO_TARGET_DIR "uwp-$kind-bundle"
    if (Test-Path $parts) { Remove-Item $parts -Recurse -Force }
    New-Item -ItemType Directory -Force $parts | Out-Null
    foreach ($cpu in $Arch) {
        & (Join-Path $tools 'makeappx.exe') pack /o /d $layouts[$cpu] /p (Join-Path $parts "$stem-$cpu.msix")
        if ($LASTEXITCODE -ne 0) { throw "makeappx failed with $LASTEXITCODE" }
    }
    $version = ([xml](Get-Content (Join-Path $root 'windows\uwp\AppxManifest.xml') -Raw -Encoding UTF8)).Package.Identity.Version
    $output = Join-Path $dist "$stem.msixbundle"
    & (Join-Path $tools 'makeappx.exe') bundle /o /bv $version /d $parts /p $output
    if ($LASTEXITCODE -ne 0) { throw "makeappx bundle failed with $LASTEXITCODE" }
}

if ($Package) {
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
    & (Join-Path $tools 'signtool.exe') sign /fd SHA256 /sha1 $certificate.Thumbprint /s My $output
    if ($LASTEXITCODE -ne 0) { throw "signtool failed with $LASTEXITCODE" }
    Export-Certificate -Cert $certificate -FilePath (Join-Path $dist 'rubi-uwp.cer') | Out-Null
    # What a GitHub release carries beside the package: the install script, and the runtime the
    # package depends on where the SDK has it for that architecture (most PCs have it already).
    Copy-Item (Join-Path $root 'windows\uwp\install.ps1') $dist -Force
    $vclibs = "${env:ProgramFiles(x86)}\Microsoft SDKs\Windows Kits\10\ExtensionSDKs\Microsoft.VCLibs\14.0\Appx\Retail"
    foreach ($cpu in $Arch) {
        $runtime = Get-ChildItem (Join-Path $vclibs $cpu) -Filter '*.appx' -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($runtime) {
            $into = Join-Path $dist "Dependencies\$cpu"
            New-Item -ItemType Directory -Force $into | Out-Null
            Copy-Item $runtime.FullName $into -Force
        }
    }
}
Write-Host "Output: $output ($([math]::Round((Get-Item $output).Length / 1MB, 2)) MB)"
