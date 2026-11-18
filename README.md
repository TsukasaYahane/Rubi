# Rubi

第三方哔哩哔哩客户端：Rust 写内核，两套界面——Android 用 Compose，Windows 用 UWP（windows-rs）。

仅供交流学习使用。本仓库的一切内容均与 B 站官方无关；正常使用请遵循《哔哩哔哩弹幕网用户使用协议》等相关用户协议，使用本软件的一切风险请自行承担。

## 下载

| 平台 | 地址 | 文件 |
|---|---|---|
| Windows | [GitHub Releases](https://github.com/TsukasaYahane/Rubi/releases/latest) | `rubi-uwp.msixbundle`（x64 + arm64 合包） |
| Windows | 微软商店 | <!--store-link-->（上架后补链接） |
| Android | [GitHub Releases](https://github.com/TsukasaYahane/Rubi/releases/latest) | `rubi-release.apk` |

只要某一个架构时，Release 里也有单独的 `rubi-uwp-arm64.msix` / x64 包。

### Windows 安装

安装包是自签名的，先信任证书再装：

1. 双击 `rubi-uwp.cer` →「安装证书」→ 位置选**本地计算机** →「将所有的证书都放入下列存储」→ **受信任的人**。
2. 双击 `rubi-uwp.msixbundle`，或在该目录运行 `install.ps1`。

不想保留证书的话，装完可以在 `certlm.msc` 的「受信任的人」里删掉 `CN=Rubi`。

商店版由微软商店签名，不需要这一步。

## 功能

- **播放**：画质按账号权限；自研弹幕渲染与过滤（规则表可编辑）；听视频（纯音频）；后台播放与系统媒体面板；`rubi://` 协议
- **内容**：推荐（权重可调，「为什么推荐」能看每个因子的贡献）、热门与排行、分区、影视、直播、竖屏推流、合集与分 P
- **社交**：登录（扫码 / 账密 / 手机号，滑块验证）、个人空间、关注动态、评论与鱼骨图、消息中心、私信
- **本地**：离线缓存与导出（音频导出为真正的 M4A）、后台下载、历史 / 稍后 / 收藏 / 播放列表
- **外观**：Metro 风格，直角、不用底色分割；三档主题（跟随系统 / 浅色 / 深色）；图形与纯文字两种皮肤；窗口宽度自适应（窄屏为四个文字标签）

## 构建

需要 Rust（workspace 用 2024 edition）。

```bash
cargo build -p rubi-cli          # 命令行，最快验证内核
cargo test --workspace --offline # 测试
```

**Android**：`scripts/build-apk.ps1 -Variant release`，产物落在 `dist/`。

**Windows**：UWP 需要固定的 nightly 与独立 target，脚本负责这些：

```powershell
scripts\build-uwp.ps1 -Register                 # 松散注册，日常开发
scripts\build-uwp.ps1 -Package -Arch x64,arm64  # 打包，自签名
scripts\build-uwp.ps1 -Store   -Arch x64,arm64  # 商店包（读 scripts\store.local.ps1）
```

商店包的预留身份不入库：`scripts\store.local.ps1` 里写 `$storeName` / `$storePublisher` / `$storeDisplayName`，缺这个文件时 `-Store` 会直接报错。

## 仓库结构

| 目录 | 内容 |
|---|---|
| `crates/model` | 数据模型 |
| `crates/api` | B 站接口与网络 |
| `crates/core` | 业务：播放、缓存、推荐、登录、评论等 |
| `crates/filter` | 内容与弹幕过滤 |
| `crates/recommend` | 推荐打分 |
| `crates/store` | SQLite 持久化 |
| `crates/ffi` | 给 Android 与 Windows 的绑定层 |
| `crates/uwp` | Windows 界面 |
| `android/` | Android 界面（Kotlin / Compose） |

## 第三方组件

安装包里链接的第三方库与它们各自的许可声明见 [`THIRD-PARTY-NOTICES.md`](THIRD-PARTY-NOTICES.md)，Apache-2.0 与 MIT 的许可正文在 [`LICENSES/`](LICENSES)。

界面用的是系统字体，图标全部由代码或字符绘制，仓库里没有内置字体与图标资源。
