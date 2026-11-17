# Rubi 安装脚本（GitHub 版）。右键「使用 PowerShell 运行」，或在本文件夹运行：
#   powershell -ExecutionPolicy Bypass -File install.ps1
# 它把同一文件夹里的 rubi-uwp.cer 导入本机「受信任的人」（需要管理员，会弹出确认），然后安装同一
# 文件夹里的 .msixbundle（或 .msix）。Dependencies\<架构>\ 下若有运行库安装包，一并安装。
# 证书只用于 Rubi 的自签名安装包；不要的话可在 certlm.msc 的「受信任的人」里删掉 CN=Rubi。
$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot

$admin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $admin) {
    Start-Process powershell -Verb RunAs -ArgumentList @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$PSCommandPath`"")
    return
}

try {
    $certificate = Join-Path $here 'rubi-uwp.cer'
    if (-not (Test-Path $certificate)) { throw "找不到证书 $certificate" }
    $package = Get-ChildItem -Path "$here\*" -Include '*.msixbundle', '*.msix' -File |
        Sort-Object { $_.Extension -ne '.msixbundle' } | Select-Object -First 1
    if (-not $package) { throw "本文件夹里没有 .msixbundle 或 .msix 安装包" }

    Write-Host "导入证书到本机「受信任的人」..."
    Import-Certificate -FilePath $certificate -CertStoreLocation Cert:\LocalMachine\TrustedPeople | Out-Null

    $arch = switch ($env:PROCESSOR_ARCHITECTURE) { 'ARM64' { 'arm64' } 'AMD64' { 'x64' } default { $env:PROCESSOR_ARCHITECTURE.ToLower() } }
    $dependencies = @(Get-ChildItem (Join-Path $here "Dependencies\$arch") -Filter '*.appx' -ErrorAction SilentlyContinue |
        ForEach-Object FullName)

    Write-Host "安装 $($package.Name) ..."
    if ($dependencies.Count) {
        Add-AppxPackage -Path $package.FullName -DependencyPath $dependencies -ForceUpdateFromAnyVersion
    } else {
        Add-AppxPackage -Path $package.FullName -ForceUpdateFromAnyVersion
    }
    Write-Host "安装完成，可在开始菜单里找到 Rubi。" -ForegroundColor Green
} catch {
    Write-Host "安装失败：$($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Message -match 'VCLibs|依赖|dependency') {
        Write-Host "缺少运行库 Microsoft.VCLibs.140.00：请先从微软商店安装任意一个应用（多数电脑已经有），或把对应架构的运行库放进 Dependencies\$arch\ 后重试。"
    }
}
Read-Host "按回车关闭"
