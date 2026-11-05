# Rubi 工作约定

## 改动代码前（强制）

动任何代码前，先按 `docs/design.md` §11.4「改动前架构检查」逐条核对，不满足的先调整设计再写：

- SOLID 五原则、合成复用原则、迪米特法则；
- 业务接口最小可见性（Rust `fn` → `pub(crate)` → `pub`；Kotlin `private` → `internal` → 公开）；
- 单个业务的接口只服务该业务，跨服务协作只经 `crates/core/src/ports.rs` 的窄接口；
- 具体实现（`HttpClient`、`SqliteStore`、`Web*`、`LiveSocket`）只在 `core::Services::open` 组装。

改完后的说明里写明本次改动对这些规则的影响（新增了哪些公开接口、给谁用）。

## 其他

- UI 设计改动先出可交互原型，审批后再写代码。
- 构建：`scripts/build-apk.ps1 -Variant release`，在 `D:\` 临时目录构建并清理。
