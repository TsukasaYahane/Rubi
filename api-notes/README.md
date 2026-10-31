# API 笔记

基线：`TsukasaYahane/bilibili-API-collect`（上游 2024-04 快照，上游已归档）。
本目录记录 Rubi 实际使用的接口、与文档的差异、以及抓包确认状态。

状态说明：
- ✅ 已对照文档或实测确认
- 🔶 按文档实现，未经实测
- ❓ 推测，需抓包确认

## 签名与会话

| 项 | 状态 | 说明 |
|---|---|---|
| WBI 签名 | ✅ | `crates/api/src/wbi.rs`，单元测试使用文档测试向量（mixin_key、w_rid） |
| `x/web-interface/nav` 取 wbi_img | 🔶 | 游客返回 -101 但仍带 `wbi_img` |
| `x/frontend/finger/spi` 取 buvid3/4 | 🔶 | |
| 扫码登录 generate / poll | 🔶 | poll 内层 code：0 成功 / 86101 未扫 / 86090 已扫未确认 / 86038 过期；cookie 来自 Set-Cookie |
| buvid 激活、dm_img 参数、-352/412 处理 | ❓ | 文档无体系，需抓包 |

## 候选来源

| 接口 | 状态 | 说明 |
|---|---|---|
| `x/web-interface/wbi/index/top/feed/rcmd` | ✅ | 2026-10-30 游客实测返回 20 条，其中 2 条 `goto=ad` |
| `x/web-interface/popular` | ✅ | 2026-10-30 游客实测 |
| `x/web-interface/ranking/v2` | ✅ | 2026-10-30 游客实测，无需 WBI |
| `x/web-interface/archive/related` | 🔶 | |
| `x/web-interface/dynamic/region` | ⛔ | 2026-10-31 已下线：任何 rid 均返回 -404「啥都木有」；偏好分区源已停用 |
| `x/web-interface/region/feed/rcmd` | 🔶 | 替代接口：`from_region` 只认 v2 主分区号（如 1005），旧 tid 与 v2 子分区返回 -400；条目无分区字段，封面为 `http://` |
| `x/polymer/web-dynamic/v1/feed/all?type=video` | 🔶 | 关注 UP 投稿，需登录 |

## 广告 / 推广识别字段（见 `crates/filter/src/default_rules.toml`）

| 规则 | 状态 |
|---|---|
| `goto` ∈ ad* | ✅ 推荐流中实测出现 `goto=ad`（卡片无标题） |
| `business_info` 非空 | ✅ 2026-10-31 抓包：广告卡 `business_info.is_ad = true`、`business_mark.text = "广告"`；普通视频卡为 `null` |
| 必火推广标记 | ❓ 网页推荐 180 条中未见（无带推广标记的 av 卡），可能只在 App 推荐流出现 |
| 商业合作稿件标记 | ❓ 未取得样本 |
| 置顶带货评论 | ✅ 2026-10-31 抓包：商品项 `content.jump_url.*.extra.goods_prefetched_cache`、正文 `b23.tv/cm-…`；`jump_url` 也用于搜索词高亮与番剧链接，不能整体判定 |
| 系统推广通知 | — 客户端未展示系统通知，暂不需要 |

## 视频与弹幕

| 接口 | 状态 | 说明 |
|---|---|---|
| `x/web-interface/view` | ✅ | 2026-10-30 实测 |
| `x/web-interface/view/detail/tag` | ❓ | 标签接口，旧的 `x/tag/archive/tags` 可能已下线 |
| `x/player/wbi/playurl` | ✅ | fnval 4048；游客实测只给到 480P 流，画质由服务端按账号权限决定，不做任何绕过 |
| `x/v2/dm/wbi/web/seg.so` | ✅ | 2026-10-30 实测解码 5499 条；字段号见 `crates/api/src/proto.rs` |
| `x/v2/dm/web/view` | ✅ | `CommandDm.mid` 实为 int64（varint），不是 string；其余字段解码正常 |

## 评论与搜索

| 接口 | 状态 | 说明 |
|---|---|---|
| `x/v2/reply/wbi/main` | ✅ | 2026-10-30 游客实测；置顶在 `top_replies` 与 `upper.top`，分页用 `pagination_str` |
| `b23.tv/<code>`（分享短链，302 `Location` 指向目标页；契约测试用 `b23.tv/<BV号>` 验证跳转） | ✅ | 不跟随跳转，只读 `Location` |
| `x/v2/reply/reply` | ✅ | 2026-10-30 实测；二级回复为扁平列表，按 `parent` 重建对话树（`rubi_model::flatten_thread`）；`data.root` 为根评论（2026-10-31 契约测试确认），用于从通知打开不在已加载页上的对话树 |
| `x/web-interface/wbi/search/type` | ✅ | 2026-10-30 游客实测；标题含 `<em>` 高亮与 `&#x27;` 等实体，已解码 |
| `s.search.bilibili.com/main/hotword` | ✅ | 2026-10-31 起用作热搜；返回体无 `data` 包装；每条带 `stat_datas.is_commercial`（字符串，样本均为 "0"，"1" 未见） |
| `x/web-interface/search/square` | ✅ | 热搜兜底；无推广标记字段 |

## 消息中心与私信（需登录，均未实测）

| 接口 | 状态 | 说明 |
|---|---|---|
| `x/msgfeed/unread` | 🔶 | reply / at / like 未读数 |
| `api.vc…/session_svr/v1/session_svr/single_unread` | 🔶 | 私信未读 = follow_unread + unfollow_unread |
| `x/msgfeed/reply` · `at` · `like` | 🔶 | 游标 `id` + `reply_time` / `at_time` / `like_time`；like 列表在 `data.total` 下；评论定位：回复 / @ 取 `item.source_id` + `item.root_id`，赞取 `item.type == "reply"` 时的 `item.item_id` |
| `api.vc…/session_svr/v1/session_svr/get_sessions` | 🔶 | 会话无昵称，另查 `account/v1/user/cards` |
| `api.vc…/svr_sync/v1/svr_sync/fetch_session_msgs` | 🔶 | `content` 是 JSON 字符串；msg_type 1 文本 / 2 图片 / 5 撤回 |
| `api.vc…/web_im/v1/web_im/send_msg` | 🔶 | 表单 `msg[...]` + `csrf` + `csrf_token`，`dev_id` 为 UUID 格式 |
| `api.vc…/session_svr/v1/session_svr/update_ack` | 🔶 | 标记已读 |
| 营销私信识别 | ✅ | 2026-10-31 抓包：`msg_type = 10` 是通知卡片，官方账号（`system_msg_type = 7`，带 `account_info`）也用；营销 = 通知卡片 + `system_msg_type = 0` + `is_follow = 0`（`can_fold = 1`） |

## UP 主页

| 接口 | 状态 | 备注 |
|---|---|---|
| `x/web-interface/card` | ✅ | 资料、粉丝、投稿数、`following`（是否已关注） |
| `x/space/wbi/arc/search` | ⚠️ | WBI 签名；`is_charging_arc` / `elec_arc_type` 标记充电专属。风控严格：连续请求时常返回 -352 或 HTTP 412（非 JSON 拦截页），客户端只提示并让用户手动重试 |

## 互动与个人库（需登录，均未实测）

| 接口 | 状态 |
|---|---|
| `x/web-interface/archive/relation` | ✅ 2026-10-31 实测 |
| `x/web-interface/archive/like` | ✅ 2026-10-31 实测（点赞 / 取消） |
| `x/v2/dm/post`（发送弹幕：type=1、oid=cid、msg、progress、mode、color、fontsize、rnd；返回 `dmid`） | ✅ 2026-10-31 实测返回成功；新弹幕不会立即出现在公开分段（审核 / 缓存） |
| `x/web-interface/archive/like/triple`（一键三连：返回 like / coin / fav / multiply 各自成败） | ✅ 2026-10-31 实测：已投过币时 `coin` 仍为成功但不再投币（`multiply` 0） |
| `x/web-interface/coin/add` | ✅ 2026-10-31 实测（`select_like=0` 时只投币） |
| `x/v3/fav/folder/created/list-all` · `x/v3/fav/resource/deal` · `x/v3/fav/resource/list` | ✅ 2026-10-31 实测（列表、加入 / 移出、浏览） |
| `x/v2/history/toview/add` · `x/v2/history/toview` · `x/v2/history/toview/del` | ✅ 2026-10-31 实测（添加、列表；移除用 `del`，客户端暂未接入） |
| `x/relation/modify` · `x/relation/followings` | 关注 / 取关 ✅ 2026-10-31 实测；关注列表 🔶 |
| `x/web-interface/history/cursor` | ✅ 2026-10-31 实测 |

需要登录的接口要等你用 `rubi login` 扫码后再实测；写操作（点赞、投币、私信等）会作用在你的真实账号上，实测前我会先问你。
