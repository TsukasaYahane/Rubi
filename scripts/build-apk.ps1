<#
.SYNOPSIS
  Builds the Rubi APK in a throwaway directory on D:\ and leaves only dist\rubi-<variant>.apk.

.DESCRIPTION
  Per the build rule (design.md §11.3): sources are copied to D:\rubi-build-<timestamp>,
  built there, and the directory is removed afterwards whether or not the build succeeds.
  Dependency caches stay in D:\rubi-cache (cargo, gradle); no build products are kept there.

.EXAMPLE
  powershell -File scripts\build-apk.ps1 -Variant release
#>
param([ValidateSet('debug', 'release')] [string]$Variant = 'debug')

$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$cache = 'D:\rubi-cache'
$work = "D:\rubi-build-$(Get-Date -Format yyyyMMdd-HHmmss)"
# Not under AppData: packaged apps (e.g. the Claude desktop app) get AppData redirected
# to a private folder, which hides the SDK from everything else.
$sdk = if ($env:ANDROID_HOME) { $env:ANDROID_HOME } else { "$env:USERPROFILE\Android\Sdk" }
$ndk = Get-ChildItem "$sdk\ndk" -Directory | Sort-Object Name | Select-Object -Last 1
$gradle = "$cache\gradle-dist\gradle-8.14.3\bin\gradle.bat"
$task = 'assemble' + $Variant.Substring(0, 1).ToUpper() + $Variant.Substring(1)

try {
    New-Item -ItemType Directory $work | Out-Null
    robocopy $repo $work /E /XD target .git .rubi-dev dist build .gradle /XF local.properties /NFL /NDL /NJH /NJS /NP | Out-Null
    if ($LASTEXITCODE -ge 8) { throw "copying sources failed (robocopy $LASTEXITCODE)" }

    $env:CARGO_HOME = "$cache\cargo"
    $env:GRADLE_USER_HOME = "$cache\gradle"
    $env:CARGO_TARGET_DIR = "$work\target"
    $env:ANDROID_HOME = $sdk
    $env:ANDROID_NDK_HOME = $ndk.FullName
    $env:JAVA_HOME = 'C:\Program Files\Java\jdk-21'
    # JDK's unix-domain pipes fail under the default temp path on this machine.
    $env:JAVA_TOOL_OPTIONS = "-Djdk.net.unixdomain.tmpdir=$cache\tmp"
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    New-Item -ItemType Directory "$cache\tmp" -Force | Out-Null

    Push-Location "$work\android"
    try {
        # In-process Kotlin compilation: no Kotlin daemon left holding files in $work.
        & $gradle $task --no-daemon --console=plain "-Pkotlin.compiler.execution.strategy=in-process"
        if ($LASTEXITCODE -ne 0) { throw "gradle $task failed ($LASTEXITCODE)" }
    } finally { Pop-Location }

    New-Item -ItemType Directory "$repo\dist" -Force | Out-Null
    $apk = Get-ChildItem "$work\android\app\build\outputs\apk\$Variant\*.apk" | Select-Object -First 1
    $out = "$repo\dist\rubi-$Variant.apk"
    Copy-Item $apk.FullName $out -Force
    $mb = [math]::Round($apk.Length / 1MB, 2)
    "APK: $out ($mb MB, budget 15 MB)"
} finally {
    Set-Location $repo
    # The forked Gradle daemon can hold file handles for a few seconds after exiting.
    for ($i = 0; $i -lt 15 -and (Test-Path $work); $i++) {
        Remove-Item $work -Recurse -Force -ErrorAction SilentlyContinue
        if (Test-Path $work) { Start-Sleep -Seconds 2 }
    }
    if (Test-Path $work) { Write-Warning "Temp dir not fully removed: $work" } else { "Cleaned up $work" }
}
