# Rubi 设计文档 v0.1

> 第三方 bilibili Android 客户端：Rust 核心 + 原生 UI，去广告与干扰组件，自建本地推荐算法。
> 本文档由需求访谈（Q1–Q36）整理，2026-10-30 定稿。后续变更请在文末「变更记录」追加。

---

## 1. 目标与边界

### 1.1 目标
- 保留 B 站主要观看功能，重构 UI（Windows Phone Metro 风格）。
- 过滤：必火推广、广告卡片、首页横幅广告/活动页、商品卡片、置顶带货评论、指令型互动弹幕；第二阶段过滤直播弹窗、礼物横幅等。
- 首页不直接使用 B 站推流池，改为多源候选 + 本地自建推荐算法。

### 1.2 非目标（明确不做）
- 任何付费相关功能：大会员专属内容、充电视频、绕过付费/画质限制。仅使用账号本身已有权限。
- 投稿、充电/购买/会员购、创作中心。（私信已于 2026-10-30 移出非目标，见消息中心）
- 自动识别或绕过验证码（极验等人机验证必须由用户手动完成）。

### 1.3 分发
- 项目代号 **Rubi**，包名 `dev.rubi.app`，名称/图标不含 bilibili 字样。
- 私有 GitHub 仓库，自用 + 少数人侧载；稳定后再决定是否公开开源（考虑到上游 API 文档库已因律师函删库）。不上架应用商店。

---

## 2. 版本规划

| 阶段 | 内容 |
|---|---|
| **MVP** | 自建推荐首页、视频播放（DASH/画质切换）、普通弹幕+过滤、搜索、UP 主页、扫码登录（可游客）、历史、收藏、稍后再看、点赞/投币/收藏、后台播放、消息中心（回复 / @我 / 赞 / 私信）、评论区（鱼骨图对话树） |
| **1.1** | 短信登录、账号密码登录（含极验 WebView、二次验证） |
| **第二阶段** | 动态、直播（观看/弹幕/礼物横幅与弹窗过滤）、离线缓存、番剧影视（非付费部分） |
| **v2 推荐** | 在线学习微调权重 |
| **待定** | 平板/折叠屏/TV 适配、纯听视频 UI、远程规则订阅、自定义调色盘 |

---

## 3. 总体架构

```
┌─────────────── Android (Kotlin) ───────────────┐
│ Jetpack Compose UI (按 feature 分模块)          │
│ Media3 / ExoPlayer + MediaSessionService        │
│ Coil (图片)   弹幕绘制层 (SurfaceView/TextureView) │
└──────────────────────┬─────────────────────────┘
                       │ UniFFI (async → suspend)
┌──────────────────────┴─────────────────────────┐
│ ffi      门面：只做类型转换，按领域分模块         │
├────────────────────────────────────────────────┤
│ core     领域服务 + 组装根（Services::open）     │
├──────────┬──────────┬────────────┬─────────────┤
│ api      │ filter   │ recommend  │ store       │
│ 协议/签名 │ 规则管线  │ 来源+因子   │ SQLite 仓储  │
│ 限流/会话 │ 弹幕处理  │ 打分/解释   │             │
└──────────┴──────────┴────────────┴─────────────┘
          Rust (tokio, reqwest + rustls)
```

### 3.1 设计原则
遵循 SOLID、合成复用原则、迪米特法则；轻量快速。
- **接口隔离**：`api` 按领域暴露 trait（`AuthApi`、`FeedApi`、`VideoApi`、`DanmakuApi`、`SearchApi`、`UserApi`、`LiveApi`…），不做大一统客户端。
- **开闭 + 合成复用**：过滤与打分都以小 trait 实现单元，经组合器拼装；新增能力 = 新增实现，不改旧代码。不使用继承式设计。
- **依赖倒置 + 迪米特**：`recommend` 只依赖 `Source` / `Factor` 抽象，不认识具体 API；Kotlin 只依赖 `ffi` 生成接口。
- 依赖一律构造函数注入（trait 对象或泛型），只在 `core::Services::open` 组装。
- **单一职责 + 功能私有化**（2026-11-01 重构）：`core` 按领域拆成独立服务——`Account`、`Home`、`Videos`、`Danmakus`、`Stein`、`Comments`、`Discover`、`Following`、`Space`、`Actions`、`History`、`WatchLater`、`Favorites`、`Messages`、`Playlists`（列表编辑）、`PlayQueue`（视频页连播）、`Listening`（听视频，三者共用 crate 内部的 `Lineup`）、`Preferences`（设置与备份）、`Emotes` 等。`api` 的账号类接口按业务拆成 `VideoActionApi`、`FavoritesApi`、`WatchLaterApi`、`HistoryApi`、`FollowingsApi`，同一个 `WebAccount` 在组装处以各自接口分别交给对应服务；直播消息通道经 `ChannelOpener` 注入（2026-11-05）。每个服务字段私有，只持有自己需要的 API trait；服务之间只经窄接口（`core/src/ports.rs`）协作：`RuleSource` / `FeedConfig`（规则表与编译好的过滤器，由 `SettingsService` 实现，规则只在改动时编译一次）、`FeedbackSink` / `HomeFeed` / `ProfileBackup`（由 `FeedService` 实现）、`VideoLookup` / `VideoStreams`（由 `Videos` 实现）、`LoginState`、`SettingsPort`、`PlaylistBackup`，服务看不到实现者的其余部分；仓储依赖用 store 的 trait，读改写在 store 内一次事务完成。卡片缓存 `CardCache`（不存原始 JSON）、评论根缓存都有上限；事件、曝光、已看表按时间 / 数量自动清理。设置在 JSON 上仍是一个平铺对象，Kotlin 只按键提交 merge patch（`patch_settings_json`），互不覆盖。（2026-11-03 架构审查后修订）
- **门面按业务分对象**（2026-11-05）：`RubiCore` 只负责打开核心，并按业务交出对象（`core.history()`、`core.favorites()`、`core.playQueue()`…）；Kotlin 的状态类只接收自己业务的对象（如 `HistoryState(HistoryService)`、`LiveState(LiveService, EsportsService)`）；页面函数同样只接收用到的业务对象与 `UiPrefs` / `CoreSettings` / `Signin`（登录状态与会话）/ `Appearance`（外观状态）；视频页与竖屏推流依赖多，按功能打包为参数对象（`ui/VideoDeps.kt`：`VideoPageDeps` 含 `actionRow: VideoActionDeps` 与 `panel: VideoPanelDeps`，下层只拿到自己那一份），只有 `RubiApp` 与组装处 `RubiRoot` 持有 `RubiCore`。
- **门面不含业务**：`ffi` 只把 `Ffi*` 记录与核心类型互转；清晰度上限（1080P）、弹幕空文本、楼中楼分页、听视频的上一首 / 下一首规则都在 `core`，Android 与将来的 iOS 共用。CLI 同样只调用服务。
- **Android 端**：页面不自写分页，统一用 `Paged`（`ui/Paging.kt`，游标泛型、去重、错误重试、重置时丢弃旧请求）；播放页拆为页面本身、`PlayerEffects`（小窗与后台、观看心跳、列表连播、弹幕分段加载）、`CommentSection`、`RelatedList`；听视频的选曲交给 core，`Listening` 只负责播放。播放相关的状态不再是全局单例：`MainPlayer`（正片 / 听视频）、`PreviewPlayer`、`Listening`、`Pip` 与全屏状态由应用持有的 `Media` 一次构建，界面经 `LocalMedia` 取用，播放服务经 Application 取用。评论区与 UP 页也用 `Paged`，可设最小翻页间隔（评论 1.5 秒、UP 投稿 1 秒）防止 412。

### 3.2 Cargo workspace

| crate | 职责 |
|---|---|
| `api` | 会话与 cookie、WBI 签名、bili_ticket、buvid、全局限流与重试、DTO、领域 trait 及实现 |
| `filter` | `trait Rule { fn judge(&self, item: &Item) -> Verdict }`、规则表加载、组合管线、过滤日志；弹幕处理（指令弹幕过滤、空降适配器、布局计算） |
| `recommend` | `trait Source`、`trait Factor`、打分器、候选池调度、推荐解释 |
| `store` | SQLite（候选池、反馈信号、画像、规则、设置），Repository trait |
| `model` | 共享领域类型（视频卡片、弹幕、反馈信号、评论树），无 I/O；让 `filter` / `recommend` 不依赖 `api` |
| `core` | 领域服务（见 3.1）与组装根 `Services`；把 API 适配成推荐来源，FeedService、弹幕加载、导出导入 |
| `ffi` | UniFFI 门面：一个 `RubiCore` 对象持有各服务，按领域分文件导出，只做类型转换 |
| `cli` | 命令行调试工具（二进制名 `rubi`）：终端查看自建推荐流、打分明细、过滤日志、弹幕处理、扫码登录 |

构建：cargo-ndk 交叉编译，Gradle 任务自动调用。

### 3.3 网络职责
- **全部 B 站 API 请求在 Rust**：签名、cookie、风控头、限流（单来源每分钟上限，全局令牌桶）、重试。
- **图片**：Kotlin Coil。
- **音视频流**：ExoPlayer 自行拉取，Rust 提供 URL 与所需 Referer/UA 等请求头。

---

## 4. API 来源与维护

- 基线：`TsukasaYahane/bilibili-API-collect`（上游 2024-04-21 快照；上游已于 2026-01 因律师函删库归档，不再更新）。
- 已知缺口：web 首页 `wbi/index/top/feed/rcmd` 未收录、广告/推广字段无文档、风控（-352/412、buvid 激活、dm_img）无体系、gRPC 仅 proto、部分互动弹幕类型未记录、直播弹幕 2023 年后的脱敏/鉴权变化未覆盖。
- 补齐方式：用户自行抓包（web：浏览器 DevTools；APP：mitmproxy / Reqable），结果维护于 `api-notes/`。
- 每个使用中的接口配**契约测试**（游客身份），校验关键字段存在。
- B 站原推荐来源优先使用 web 端 `wbi/index/top/feed/rcmd`（WBI 签名，风险低于 APP access_key 方案）。

---

## 5. 登录与凭证

| 方式 | 阶段 | 说明 |
|---|---|---|
| 游客 | MVP | 所有只读功能可用，便于调试与防风控 |
| 扫码登录 | MVP | web 版 passport qrcode generate/poll |
| 短信登录 | 1.1 | 极验 WebView 由用户手动完成 → 回传 Rust 继续流程 |
| 账号密码 | 1.1 | 先取公钥 RSA 加密密码；风控二次验证（手机验证）分支流程，用户手动完成 |

- **不保存密码**。仅保存 cookie / refresh_token（Android Keystore 加密）；手机号可选记住（同样加密）。
- 实现 cookie 刷新（correspondPath / refresh_csrf）。
- 凭证绝不上传、绝不进入导出文件。

---

## 6. 过滤系统

### 6.1 规则表
- 规则以带版本号的 TOML/JSON 存储，可导入导出；预留「规则来源」字段（v1 不做远程订阅）。
- 内置开关：逐项控制以下各类组件。
- 用户自定义：关键词/正则（标题、标签、简介）、UP 黑名单、分区屏蔽、时长区间、发布时间区间。
- 被拦截项写入过滤日志，提供「被过滤内容」调试页核对误杀。

### 6.2 内容过滤

| 对象 | 默认处理 |
|---|---|
| 必火推广视频 | 隐藏 |
| 广告视频卡片 | 隐藏 |
| 首页横幅广告、活动页 | 隐藏 |
| 视频页商品卡片 | 隐藏 |
| 评论区置顶带货评论 | 隐藏 |
| UP 主商业合作稿件 | **不惩罚**，正常参与推荐 |

广告/推广识别（无官方文档，多层启发式，均为规则表条目）：
1. 卡片类型 / `goto` 非普通视频（ad、ad_av、live、game、推广等）；
2. 广告字段存在（如 `is_ad`、`ad_info`、`creative_id`，待抓包确认）；
3. 无 bvid/aid，或跳转为外链/商品链接；
4. 必火标记（角标/字段，待抓包确认）。

### 6.3 弹幕过滤
- 指令型互动弹幕（投票、三连/关注按钮、预约、评分、链接卡片、`#UP#` `#LINK#` `#ATTENTION#` 等）：**全部过滤**。
- 高级弹幕（mode 7）：保留，单独开关。
- 常规设置：关键词/正则、屏蔽用户、密度、防遮挡。

### 6.4 空降弹幕适配器
识别：
- **带关键字**：「空降 / 空降到 / 跳到 + 时间」，时间可在任意位置；不误伤无时间的「空降成功」等。
- **纯时间戳**：须同时满足——
  1. 整条仅为时间戳（允许空白、标点、括号），如 `05:32`、`5:32。`、`【05:32】`、`5分32秒`；
  2. 目标时间晚于该弹幕出现时间 ≥10 秒（回指不算）；
  3. 目标时间 ≤ 视频总时长。
- 时间格式：`mm:ss`、`hh:mm:ss`、`x分y秒`、`xx.xx`。

处理：
- 两类合并，按目标时间 ±5 秒聚类，每簇仅保留**最早发送**的一条，作为普通弹幕显示。
- 进度条显示可点击「空降点」标记，点击跳转；设置中可关闭。
- 两类识别各自有基于真实弹幕样本的回归单元测试。

### 6.5 直播（第二阶段）
- 基于直播信息流 WebSocket（16 字节包头，zlib/brotli）按 cmd 过滤：礼物横幅（SEND_GIFT、COMBO_SEND、WIDGET_BANNER…）、弹窗与提示（USER_TOAST_MSG、NOTICE_MSG、ENTRY_EFFECT、红包等）。具体清单开发时确定。

---

## 7. 推荐系统

### 7.1 运行位置
完全本地（Rust 核心 + SQLite）。

### 7.2 候选来源（加权混合，权重可调）
- B 站原推荐（仅作候选之一）
- 关注 UP 新投稿
- 热门 / 排行榜 / 入站必刷
- 偏好分区最新
- 已看视频的相关推荐
- 关键词/标签订阅搜索

所有候选先经过滤层（广告、推广、黑名单、关键词、分区、时长、已看），再进入排序。

### 7.3 候选池更新
- 下拉刷新或候选不足时按需拉取，写入 SQLite 候选池（带过期时间），统一打分出流。
- 全局限流防风控；v1 不做后台定时预取。

### 7.4 打分因子
| 因子 | 方向 | 说明 |
|---|---|---|
| 兴趣匹配 | + | 标签 / 分区 / UP 三层画像 + 标题分词关键词 |
| 关注 UP | + | |
| 新鲜度 | + | 发布时间衰减 |
| 话题度 | + | 单位时间播放增速；多来源同时出现加分 |
| 内容质量 | + | 赞/播、币/播、藏/播、弹幕与评论密度，按分区与发布时长归一化 |
| 探索 | + | 新领域/新 UP 的探索奖励 |
| 不感兴趣 / 不喜欢 | − | 来自显式反馈的 UP、分区、标签、标题类型 |
| 重复曝光 | − | 按同一视频展示次数递增，被点击后清零 |

- **v1**：可解释线性加权 `score = Σ wᵢ · normᵢ(x)`，权重可调；每张卡片可查看「为什么推荐」。
- **v2**：在线学习（逻辑回归 / bandit）基于反馈微调权重，手调权重作先验。
- v1 不使用端上大模型或 embedding。

### 7.5 反馈信号
- 显式：不感兴趣（原因：UP / 分区 / 标签 / 标题类型）、喜欢、点赞、投币、收藏、关注。
- 隐式：曝光未点击、快速退出、完成率、重看、搜索词。
- 全部带时间衰减（兴趣半衰期约 30 天），仅存本地。

### 7.6 导出 / 导入
单个 zip（内含 JSON/TOML）：设置、过滤规则、推荐权重、兴趣画像（种子）、可选的观看与反馈历史。**不含登录凭证**，换设备后重新登录。

---

## 8. 播放与弹幕渲染

- Media3 / ExoPlayer，运行于 `MediaSessionService`：锁屏/通知栏控制、耳机按键。后台播放为设置项「后台播放」，默认关闭（切到后台暂停）；离开播放页即停止。
- 纯音频（听视频）模式：预留仅拉音频流的接口，暂不做 UI。
- 画质：遵循账号权限（720P60 / 1080P 需登录；更高需大会员，不绕过）。
- 弹幕渲染（自研）：Rust 解析 protobuf 弹幕、过滤、轨道与碰撞布局；Kotlin 在透明 SurfaceView/TextureView 上按时间轴绘制。**兼容性与适配性优先**，性能与表现后续优化。

---

## 9. UI 设计

### 9.1 风格
- Windows Phone Metro：扁平，不做或少做圆角，**不用方框、阴影、凸起做强制分隔**；以字号、字重、留白与网格对齐区分层级。
- ASCII 与几何美学：几何/字符图标（`▶ ✕ ≡ ↻` 等）、等宽数字（播放量、时长、时间戳）、细线与网格。
- Pivot 式大号小写标题，可左右滑动切换子页；WP 风格翻转 / 滑入动效。
- 字体：Selawik 或 Inter（拉丁）、思源黑体 / Noto Sans SC（中文）、JetBrains Mono（等宽）。
  - App 实现（2026-10-30）：暂用系统字体（系统无衬线 + 系统等宽）。内置中文字体会使 APK 超出 15 MB 预算；是否改用可下载字体待定。
- 色彩：黑/白底 + 强调色，默认 **cobalt**；自定义调色盘后续再做。深浅色均支持。

### 9.2 导航与页面
- 底部导航：**首页 / 关注 / 我的**；搜索与**消息中心**入口整合于顶部栏（消息图标带未读数）。直播入口第二阶段加入。
- 消息中心：Pivot 子页「回复 / @我 / 赞 / 私信」；私信可进入对话窗口收发。过滤系统推广通知与营销私信。
- 播放页评论区：与「推荐」以 Pivot 切换。以**鱼骨图**呈现对话树——主干细线连接根评论下的全部回复，每条回复横向分支，回复的回复逐级缩进；每棵对话树默认显示 3 条，可「再看 5 条」或折叠 / 展开；支持按热度 / 按时间排序。树结构由评论的 `parent` 关系重建（二级回复列表为扁平结构，需按 parent 还原层级）。
- 首页信息流：双列卡片，或可选单列大图。
- 大图预览：卡片可见 >70% 且停留 0.8 秒后自动预览，360P、静音、带弹幕，同时仅一个；默认仅 Wi-Fi。设置可调：开关、音量、弹幕、是否允许流量预览。预览的 playurl 请求计入全局限流。

### 9.3 设计流程
1. 先产出 HTML 可交互原型（412×915，mock 数据）：首页（双列/单列切换）、播放页、关注（Pivot）、我的、搜索、设置、「为什么推荐」浮层、「被过滤内容」调试页。
2. 原型存于 `design/`，并发布为私有 Artifact 供手机审阅。
3. 逐页审计定稿后，再写 Compose 代码。

---

## 10. 质量目标（CI 校验）

| 指标 | 目标 |
|---|---|
| APK 体积 | ≤ 15 MB（arm64 so + R8） |
| 冷启动至首页首屏内容 | ≤ 1.5 s（中端机） |
| 500 候选打分排序 | ≤ 20 ms |
| 常驻内存（不含播放器缓冲） | ≤ 200 MB |

设备：Android 8.0（API 26）+，仅 arm64-v8a；手机竖屏，横屏仅全屏播放。

---

## 11. 工程与流程

### 11.1 开发顺序
1. Rust 核心（`api`：登录/WBI/feed/playurl/弹幕；`filter`；`recommend` v1；`store`）+ `cli` 调试工具 —— 与 HTML 原型**并行**。
2. HTML 原型审阅迭代。
3. 原型定稿后：Compose UI + Media3 播放。

### 11.2 版本管理与 CI
- `I:\Rubi` 初始化 git，推送私有 GitHub 仓库。
- GitHub Actions：
  - 每次 push：`cargo fmt --check`、`clippy`、`test`、性能/体积预算检查、debug APK 构建。
  - 每晚：契约测试（仅游客身份），字段变化自动开 issue。
- CI 中不存放任何账号凭证。

### 11.3 本地构建

运行 `powershell -File scripts\build-apk.ps1 [-Variant release]`，产物在 `dist\`（不入库）。

工具链与镜像（本机，2026-10-30）：Android SDK 在 `%USERPROFILE%\Android\Sdk`（platform 36、build-tools 36.1.0、NDK r29；不放 AppData，打包应用会把 AppData 重定向到私有目录），由腾讯云镜像下载；adb 在其 `platform-tools` 下；Gradle 8.14.3 在 `D:\rubi-cache\gradle-dist`；Maven 依赖经 `D:\rubi-cache\gradle\init.d` 的阿里云镜像，crates 经 `D:\rubi-cache\cargo\config.toml` 的 USTC 镜像。镜像配置只在本机缓存目录，不入库。


- 每次测试构建在 `D:\` 下新建唯一临时目录（如 `D:\rubi-build-<时间戳>`），复制源码后构建。
- 结束后（无论成败）删除该临时目录的全部构建产物并核验无残留。
- 依赖下载缓存保留在固定目录 `D:\rubi-cache`（`CARGO_HOME`、`GRADLE_USER_HOME` 指向此处），不存放构建产物。

### 11.4 改动前架构检查（强制，2026-11-04 起）

每次动代码前先过一遍下表，不满足的先调整设计再写；评审（自审或交给另一个 agent 只读复核）也按这张表逐条给结论。

| 规则 | 在 Rubi 里的具体要求 |
|---|---|
| 单一职责 | 一个 `core` 服务只管一个业务；一个 Kotlin 页面只负责呈现，取数、乐观更新、重试放到状态持有者（`*State` 类）里。文件超过约 500 行时检查是否混了多个职责 |
| 开闭 | 新能力 = 新实现（新 trait 实现、新 `Source`/`Factor`/规则单元），不在旧实现里加分支 |
| 里氏替换 | trait 的每个实现（含测试用 Fake）都满足 trait 文档的约定，不靠调用方判断具体类型 |
| 接口隔离 | `api` trait 按业务拆，调用方只拿到自己用的方法；一个 trait 的方法分属多个服务时就拆开。不做大一统客户端 |
| 依赖倒置 | 服务只依赖 `api` trait、`ports.rs` 窄接口和 store trait；具体类型（`HttpClient`、`SqliteStore`、`Web*`、`LiveSocket`）只在 `Services::open` 出现 |
| 合成复用 | 用组合与注入拼装能力，不用继承；Kotlin 不建 `Base*Screen` 之类的基类 |
| 迪米特 | 只和直接协作者说话：服务不经一个服务去拿另一个服务；Kotlin 页面拿到需要的那一个对象（服务 / 状态 / 回调），不传整个 `RubiApp` 再一路点下去 |
| 接口私有化 | 默认最小可见性：Rust 先 `fn`，再 `pub(crate)`，确实跨 crate 才 `pub`；Kotlin 先 `private`，再 `internal`。新增的 `pub`/公开 Kotlin 声明要能说出是谁在用 |
| 接口专用 | 某个业务的接口只服务该业务：不为图方便复用另一个业务的方法，需要时新开窄接口（加进 `ports.rs`） |

另外：`ffi` 只做类型转换不含业务；新增业务规则写在 `core`，Android 与 CLI 共用。

### 11.5 Windows 移植（2026-11-05 定）

路线：先 UWP，后 WinUI。核心与界面都用 Rust（`windows-rs`）。设备：桌面、Surface 为运行目标；Windows 手机没有真机，本阶段只验证编译，运行留到模拟器。分发：MSIX 自签名，不计划上架商店。视觉沿用 Metro（钴蓝、直角、图形 / 文字两套风格），窗口标题栏、滚动条、右键菜单和无障碍用系统原生。

前置工作分四步，每步汇报，走不通时停下等决定，不自行切换退路：

1. 核心编译并运行在 `x86_64-uwp-windows-msvc`。
2. 核心编译到 `thumbv7a-uwp-windows-msvc`。退路：`api` 接口注入 WinRT 的网络与 TLS 实现；仍不行则手机标为不支持。
3. 纯 Rust 跑通 XAML 窗口：代码创建控件，样式用 `XamlReader` 加载。退路：薄 C++/WinRT 外壳，只放 XAML 与绑定。
4. 最小样板：扫码登录、首页、播放一个视频。

第 3 步后出窄 / 中 / 宽三档原型（断点 600 / 840，与 Android 一致；首页、播放页、竖屏推流、我的 / 设置），审批后才写正式界面。

技术决定：最低系统 15063、目标 22621；`crates/uwp` 进 workspace 但不进 `default-members`，直接依赖 `rubi-core`，`rubi-ffi` 只服务 Android；清单放 `windows/uwp/`，构建脚本 `scripts/build-uwp.ps1`，证书不入库；为 UWP 固定一个 nightly 版本。播放用 `MediaPlayerElement` + `AdaptiveMediaSource`，Rust 本地生成 MPD 并带请求头，退路为本机回环代理，先只用 AVC，上限 1080P，样板不含弹幕。登录先只做扫码，会话存 `PasswordVault`，数据放 `LocalFolder`。页面状态放 `crates/uwp` 的 `state` 模块且不碰 XAML 类型，做 WinUI 时再抽成共享 crate。

验证记录（2026-11-05，nightly-2026-10-15，`-Z build-std`）：

- `x86_64-uwp`：`rubi-core` 编译通过，core 与 store 的 45 个单元测试通过（在包外运行，需 `Microsoft.VCLibs` 的 `vcruntime140_app.dll`；尚未在 AppContainer 内验证，也未验证联网）。
- `thumbv7a-uwp`（手机）：不通过。`ring` 的 ARM 汇编无法用 MSVC 工具链构建（LNK1181）；`windows-sys` 0.61 缺 ARM32 类型（经 `mio` / `tokio net` 依赖）。`rubi-store`、`rubi-filter`、`rubi-recommend` 能编译。
- `aarch64-uwp`（ARM 版 Surface）：`rubi-uwp` 编译并链接通过（需要 clang，LLVM 23.1.5），未实际运行。
- 第 4 步（x64，沙盒内）：首页推荐网格、三档窗口切换（520 / 720 / 1200 宽分别 2 / 3 / 4 列，导航为底栏 / 图标栏 / 图标加文字栏）、返回、二维码显示均已验证。播放已验证：系统 `AdaptiveMediaSource` 接受本地生成的 MPD，位置按实时前进。`Stream` 新增 `segment`（`SegmentBase` 字节范围）供 MPD 使用。扫码登录已验证，重启后经 `PasswordVault` 恢复会话。声音已确认（用户收听，系统音频会话电平也有输出）。
- 第 3 步（x64，已注册为 UWP 包在 AppContainer 内运行）：纯 Rust XAML 成立。`windows` crate 自 0.38 起不含 `Windows.UI.Xaml`，改用 `windows-bindgen` 0.100 从 SDK 元数据生成绑定（`tools/uwp-bindgen`，`--minimal --flat --compose`，输出 `crates/uwp/src/xaml.rs`）。`Application` 经 `compose` 继承；布局和模板用 `XamlReader` 加载；5000 行 `ListView` 只实例化约 95 行，用 `ContainerContentChanging` 填行代替数据绑定；SQLite、TLS、联网和 UI 线程回调都在沙盒内通过。

正式界面进度（2026-11-05）：播放页三档布局、详情、推荐、评论、操作按钮、弹幕显示与开关、发弹幕栏已完成。弹幕用 72 个文字块的固定池，每帧按播放位置移动，核心负责分轨。账号写操作与全屏弹幕已由用户验证。全屏由应用自管（不用系统全窗），导航栏和右栏可拖动调整，内容实时重排，宽度存在 `panes.txt`。

直播（2026-11-06）：系统 `AdaptiveMediaSource` 接受 B 站 HLS 地址，经带请求头的 `HttpClient` 下载；FLV 不用。弹幕复用播放页的 `Overlay`，时钟换成进房后的时长。

关注的直播：`xlive/web-ucenter/user/following` 只给谁在播，封面、人气、分区为空（2026-11-06 实测），开播的房间再逐个查 `getInfoByRoom`，每页最多 12 个。`xfetter/GetWebList` 按第三方资料的字段读不到数据，未使用。

我的 / 设置（2026-11-06）：核心设置按键提交 merge patch，与 Android 相同；只属于 UWP 的界面风格、失焦暂停、键盘快捷键存 `prefs.txt`。原型里的“窗口最小化时用画中画”未做：最小化后的 UWP 窗口无法进入紧凑视图。

---

## 12. 待抓包确认事项
- [ ] web `wbi/index/top/feed/rcmd` 请求参数与响应结构
- [ ] 广告卡片字段（`is_ad`、`ad_info`、`creative_id` 等）
- [ ] 必火推广标记
- [ ] 首页横幅 / 活动页数据来源
- [ ] 视频页商品卡片、置顶带货评论标记
- [ ] 未记录的指令型互动弹幕类型（投票等）
- [ ] 风控参数（buvid 激活、dm_img、-352/412 处理）
- [ ] 短信 / 密码登录 2024 年后的接口变化
- [ ] 直播信息流 2023 年后的鉴权与脱敏变化
- [ ] 消息中心（回复 / @ / 赞 / 未读数）与私信收发接口现状，系统推广通知的识别字段
- [ ] 评论二级回复与对话链接口（`parent`、`dialog` 字段）是否足以重建完整对话树

---

## 变更记录
- 2026-10-30：v0.1 初稿（访谈 Q1–Q36）。
- 2026-10-30：新增消息中心（含私信），纳入 MVP，入口在顶部栏；HTML 原型位于 `design/project/`。
- 2026-10-30：评论区（鱼骨图对话树）从第二阶段提前到 MVP。
