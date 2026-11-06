//! 直播: room lists and areas, a room's header and streams, its message channel's address,
//! and what the channel's messages mean ([`event_of`]).

use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::{LiveArea, LiveEvent, LivePlay, LiveProtocol, LiveRoom, LiveStream, LiveSubArea, now_secs};
use serde_json::Value;

use crate::error::{ApiError, Result};
use crate::http::{HttpClient, UA};
use crate::params;
use crate::parse::{image_of, str_of, u64_of};

const REC_LIST: &str = "https://api.live.bilibili.com/xlive/web-interface/v1/webMain/getMoreRecList";
const AREAS: &str = "https://api.live.bilibili.com/room/v1/Area/getList";
/// The newer `second/getList` wants a token the web page is issued, so the plain list is used.
const AREA_ROOMS: &str = "https://api.live.bilibili.com/room/v3/area/getRoomList";
const AREA_PAGE: u32 = 30;
const ROOM_INFO: &str = "https://api.live.bilibili.com/xlive/web-room/v1/index/getInfoByRoom";
const PLAY_INFO: &str = "https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo";
const DANMU_INFO: &str = "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo";
const SEND: &str = "https://api.live.bilibili.com/msg/send";
const FOLLOWED: &str = "https://api.live.bilibili.com/xlive/web-ucenter/user/following";
const LIVE_REFERER: &str = "https://live.bilibili.com/";
/// Followed rooms asked for at a time.
const FOLLOWED_PAGE: u32 = 20;
/// Used when the room's own host list is empty.
const FALLBACK_HOST: &str = "wss://broadcastlv.chat.bilibili.com:443/sub";
/// 原画, asked for first; the answer says what the room really offers.
pub const BEST_LIVE_QN: u32 = 10000;

/// Where a room's message channel lives and the key to join it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DanmuServer {
    pub token: String,
    /// `wss://host:port/sub` addresses, to try in order.
    pub urls: Vec<String>,
}

#[async_trait]
pub trait LiveApi: Send + Sync {
    /// The 推荐 rooms of the live home page (one batch; asking again gives a new one).
    async fn recommended(&self) -> Result<Vec<LiveRoom>>;
    async fn areas(&self) -> Result<Vec<LiveArea>>;
    /// One page of a parent area's rooms (`area` 0 for all of them); also whether more follow.
    async fn area_rooms(&self, parent: u32, area: u32, page: u32) -> Result<(Vec<LiveRoom>, bool)>;
    /// One page of followed UPs' rooms, those on air first; also whether more follow. Login required.
    async fn followed_rooms(&self, page: u32) -> Result<(Vec<LiveRoom>, bool)>;
    async fn room(&self, room_id: u64) -> Result<LiveRoom>;
    async fn play(&self, room_id: u64, qn: u32) -> Result<LivePlay>;
    async fn danmu_server(&self, room_id: u64) -> Result<DanmuServer>;
    /// Sends a danmaku to a room (needs login).
    async fn send(&self, room_id: u64, text: &str) -> Result<()>;
    /// The identity to join a message channel with: uid (0 for guests) and buvid3.
    fn identity(&self) -> (u64, String);
    /// Headers the player must send when fetching live streams.
    fn stream_headers(&self) -> Vec<(String, String)>;
}

pub struct WebLive {
    http: Arc<HttpClient>,
}

impl WebLive {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl LiveApi for WebLive {
    async fn recommended(&self) -> Result<Vec<LiveRoom>> {
        let d = self.http.get_json(REC_LIST, params!["platform" => "web"], false).await?;
        Ok(rooms_in(&d["recommend_room_list"]))
    }

    async fn areas(&self) -> Result<Vec<LiveArea>> {
        let d = self.http.get_json(AREAS, params![], false).await?;
        Ok(areas_of(&d))
    }

    async fn area_rooms(&self, parent: u32, area: u32, page: u32) -> Result<(Vec<LiveRoom>, bool)> {
        let d = self
            .http
            .get_json(
                AREA_ROOMS,
                params![
                    "platform" => "web", "parent_area_id" => parent, "area_id" => area, "page" => page,
                    "page_size" => AREA_PAGE, "sort_type" => "online",
                ],
                false,
            )
            .await?;
        let rooms = rooms_in(&d["list"]);
        let more = !rooms.is_empty() && u64_of(&d["count"]) > u64::from(page * AREA_PAGE);
        Ok((rooms, more))
    }

    async fn followed_rooms(&self, page: u32) -> Result<(Vec<LiveRoom>, bool)> {
        let d = self
            .http
            .get_json(FOLLOWED, params!["page" => page, "page_size" => FOLLOWED_PAGE, "ignoreRecord" => 1], false)
            .await?;
        let rooms: Vec<LiveRoom> = d["list"]
            .as_array()
            .map(|l| l.iter().map(followed_room).filter(|r| r.room_id > 0).collect())
            .unwrap_or_default();
        let more = u64::from(page) < u64_of(&d["totalPage"]);
        Ok((rooms, more))
    }

    async fn room(&self, room_id: u64) -> Result<LiveRoom> {
        let d = self.http.get_json(ROOM_INFO, params!["room_id" => room_id], true).await?;
        let room = room_of_info(&d);
        if room.room_id == 0 {
            return Err(ApiError::Decode(format!("room {room_id} has no room_info")));
        }
        Ok(room)
    }

    async fn play(&self, room_id: u64, qn: u32) -> Result<LivePlay> {
        let d = self
            .http
            .get_json(
                PLAY_INFO,
                params![
                    "room_id" => room_id, "protocol" => "0,1", "format" => "0,1,2", "codec" => "0",
                    "qn" => qn, "platform" => "web", "ptype" => 8,
                ],
                false,
            )
            .await?;
        Ok(play_of(&d))
    }

    async fn danmu_server(&self, room_id: u64) -> Result<DanmuServer> {
        let d = self.http.get_json(DANMU_INFO, params!["id" => room_id, "type" => 0], true).await?;
        Ok(danmu_server_of(&d))
    }

    async fn send(&self, room_id: u64, text: &str) -> Result<()> {
        let csrf = self.http.session().csrf().map(str::to_owned).ok_or(ApiError::NotLoggedIn)?;
        let body = self
            .http
            .post_form_from(
                SEND,
                &format!("{LIVE_REFERER}{room_id}"),
                params![
                    "bubble" => 0, "msg" => text, "color" => 0xFF_FFFF, "mode" => 1, "room_type" => 0,
                    "jumpfrom" => 0, "reply_mid" => 0, "reply_attr" => 0, "replay_dmid" => "", "statistics" => "{\"appId\":100,\"platform\":5}",
                    "fontsize" => 25, "rnd" => now_secs(), "roomid" => room_id, "csrf_token" => csrf,
                ],
            )
            .await?;
        refusal_of(body["message"].as_str().unwrap_or_default()).map_or(Ok(()), |why| Err(ApiError::from_code(-1, why)))
    }

    fn identity(&self) -> (u64, String) {
        let s = self.http.session();
        (s.dede_user_id.as_deref().and_then(|u| u.parse().ok()).unwrap_or(0), s.buvid3.unwrap_or_default())
    }

    fn stream_headers(&self) -> Vec<(String, String)> {
        vec![("Referer".into(), LIVE_REFERER.into()), ("User-Agent".into(), UA.into())]
    }
}

/// Why a room turned a danmaku down: it answers success, with a letter or a line in `message`.
fn refusal_of(message: &str) -> Option<String> {
    match message {
        "" | "0" | "ok" => None,
        "f" => Some("弹幕被系统屏蔽".into()),
        "k" => Some("弹幕被房间屏蔽".into()),
        other => Some(other.to_owned()),
    }
}

fn rooms_in(list: &Value) -> Vec<LiveRoom> {
    list.as_array().map(|l| l.iter().map(room_of).filter(|r| r.room_id > 0).collect()).unwrap_or_default()
}

/// A room as the list endpoints write it.
pub(crate) fn room_of(v: &Value) -> LiveRoom {
    let online = u64_of(&v["online"]).max(u64_of(&v["watched_show"]["num"]));
    LiveRoom {
        room_id: u64_of(&v["roomid"]).max(u64_of(&v["room_id"])),
        title: str_of(&v["title"]),
        cover: image_of(if v["cover"].is_string() { &v["cover"] } else { &v["user_cover"] }),
        uid: u64_of(&v["uid"]),
        uname: str_of(&v["uname"]),
        face: image_of(&v["face"]),
        online,
        area: Some(str_of(&v["area_v2_name"])).filter(|s| !s.is_empty()).unwrap_or_else(|| str_of(&v["area_name"])),
        // Lists only carry rooms that are on air.
        live: v.get("live_status").is_none_or(|s| u64_of(s) == 1),
    }
}

/// A room in the followed list, which says who and whether on air but has no cover or viewers.
fn followed_room(v: &Value) -> LiveRoom {
    let room = room_of(v);
    LiveRoom { cover: image_of(&v["face"]), area: String::new(), online: 0, ..room }
}

/// `getInfoByRoom`: the room header plus its anchor.
pub(crate) fn room_of_info(d: &Value) -> LiveRoom {
    let mut room = room_of(&d["room_info"]);
    let base = &d["anchor_info"]["base_info"];
    room.uname = str_of(&base["uname"]);
    room.face = image_of(&base["face"]);
    room.online = room.online.max(u64_of(&d["watched_show"]["num"]));
    room
}

pub(crate) fn areas_of(d: &Value) -> Vec<LiveArea> {
    let Some(list) = d.as_array() else { return Vec::new() };
    list.iter()
        .map(|a| LiveArea {
            id: u64_of(&a["id"]) as u32,
            name: str_of(&a["name"]),
            children: a["list"]
                .as_array()
                .map(|l| {
                    l.iter()
                        .map(|s| LiveSubArea {
                            id: u64_of(&s["id"]) as u32,
                            parent_id: u64_of(&s["parent_id"]) as u32,
                            name: str_of(&s["name"]),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
        .filter(|a| a.id > 0)
        .collect()
}

/// Streams of `getRoomPlayInfo`, AVC only (HEVC in FLV does not play, and many devices lack a
/// HEVC decoder): HLS fMP4, then HLS TS, then FLV, each over every host offered.
pub(crate) fn play_of(d: &Value) -> LivePlay {
    let playurl = &d["playurl_info"]["playurl"];
    let names: Vec<(u32, String)> = playurl["g_qn_desc"]
        .as_array()
        .map(|l| l.iter().map(|q| (u64_of(&q["qn"]) as u32, str_of(&q["desc"]))).collect())
        .unwrap_or_default();
    let mut found: Vec<(u8, LiveStream)> = Vec::new();
    let mut qn = 0;
    let mut accept: Vec<u32> = Vec::new();
    for stream in playurl["stream"].as_array().into_iter().flatten() {
        let protocol = match stream["protocol_name"].as_str() {
            Some("http_hls") => LiveProtocol::Hls,
            Some("http_stream") => LiveProtocol::Flv,
            _ => continue,
        };
        for format in stream["format"].as_array().into_iter().flatten() {
            let rank = match (protocol, format["format_name"].as_str()) {
                (LiveProtocol::Hls, Some("fmp4")) => 0,
                (LiveProtocol::Hls, Some("ts")) => 1,
                (LiveProtocol::Flv, Some("flv")) => 2,
                _ => continue,
            };
            for codec in format["codec"].as_array().into_iter().flatten() {
                if codec["codec_name"].as_str() != Some("avc") {
                    continue;
                }
                qn = qn.max(u64_of(&codec["current_qn"]) as u32);
                for q in codec["accept_qn"].as_array().into_iter().flatten() {
                    let q = u64_of(q) as u32;
                    if !accept.contains(&q) {
                        accept.push(q);
                    }
                }
                let base = str_of(&codec["base_url"]);
                for host in codec["url_info"].as_array().into_iter().flatten() {
                    let url = format!("{}{}{}", str_of(&host["host"]), base, str_of(&host["extra"]));
                    found.push((rank, LiveStream { url, protocol }));
                }
            }
        }
    }
    found.sort_by_key(|(rank, _)| *rank);
    accept.sort_unstable_by(|a, b| b.cmp(a));
    let qualities = accept
        .into_iter()
        .map(|q| (q, names.iter().find(|(n, _)| *n == q).map(|(_, d)| d.clone()).unwrap_or_else(|| q.to_string())))
        .collect();
    LivePlay {
        live: u64_of(&d["live_status"]) == 1,
        qn,
        qualities,
        streams: found.into_iter().map(|(_, s)| s).collect(),
    }
}

pub(crate) fn danmu_server_of(d: &Value) -> DanmuServer {
    let mut urls: Vec<String> = d["host_list"]
        .as_array()
        .map(|l| {
            l.iter()
                .filter_map(|h| {
                    let host = h["host"].as_str().filter(|s| !s.is_empty())?;
                    let port = Some(u64_of(&h["wss_port"])).filter(|p| *p > 0).unwrap_or(443);
                    Some(format!("wss://{host}:{port}/sub"))
                })
                .collect()
        })
        .unwrap_or_default();
    urls.push(FALLBACK_HOST.into());
    DanmuServer { token: str_of(&d["token"]), urls }
}

/// What one channel message means for the room page; `None` for everything it does not show
/// (rank changes, PK, notices…).
pub fn event_of(msg: &Value) -> Option<LiveEvent> {
    let cmd = msg["cmd"].as_str()?;
    // `DANMU_MSG:4:0:2:2:2:0` and the like carry the same body.
    let cmd = cmd.split(':').next().unwrap_or(cmd);
    let d = &msg["data"];
    match cmd {
        "DANMU_MSG" => {
            let info = &msg["info"];
            let uname = str_of(&info[2][1]);
            let text = str_of(&info[1]);
            // info[0][9]: danmaku sent by joining a 天选 lottery; info[0][12] 1: a sticker.
            if u64_of(&info[0][9]) != 0 {
                return Some(LiveEvent::Popup { text: format!("{uname}：{text}") });
            }
            if u64_of(&info[0][12]) == 1 {
                return Some(LiveEvent::Effect { text: format!("{uname}：{text}") });
            }
            Some(LiveEvent::Danmaku { uname, text, color: u64_of(&info[0][3]) as u32 })
        }
        "SEND_GIFT" => Some(LiveEvent::Gift {
            text: format!(
                "{} {} {} ×{}",
                str_of(&d["uname"]),
                str_of(&d["action"]),
                str_of(&d["giftName"]),
                u64_of(&d["num"])
            ),
        }),
        "COMBO_SEND" => Some(LiveEvent::Gift {
            text: format!(
                "{} {} {} ×{}",
                str_of(&d["uname"]),
                str_of(&d["action"]),
                str_of(&d["gift_name"]),
                u64_of(&d["total_num"]).max(u64_of(&d["combo_num"]))
            ),
        }),
        "GUARD_BUY" => {
            Some(LiveEvent::Gift { text: format!("{} 开通了 {}", str_of(&d["username"]), str_of(&d["gift_name"])) })
        }
        "USER_TOAST_MSG" | "SPECIAL_GIFT" | "GIFT_STAR_PROCESS" => {
            Some(LiveEvent::Gift { text: plain(&first_text(d, &["toast_msg", "tip", "content"])) })
        }
        "SUPER_CHAT_MESSAGE" => Some(LiveEvent::Effect {
            text: format!(
                "醒目留言 ¥{} {}：{}",
                u64_of(&d["price"]),
                str_of(&d["user_info"]["uname"]),
                str_of(&d["message"])
            ),
        }),
        "ENTRY_EFFECT" | "ENTRY_EFFECT_MUST_RECEIVE" => {
            Some(LiveEvent::Effect { text: plain(&str_of(&d["copy_writing"])) })
        }
        "INTERACT_WORD" => {
            let verb = if u64_of(&d["msg_type"]) == 2 { "关注了直播间" } else { "进入直播间" };
            Some(LiveEvent::Effect { text: format!("{} {verb}", str_of(&d["uname"])) })
        }
        "ANCHOR_LOT_START" => Some(LiveEvent::Popup {
            text: format!("天选时刻：{}（{}）", str_of(&d["award_name"]), str_of(&d["require_text"])),
        }),
        "POPULARITY_RED_POCKET_START" | "POPULARITY_RED_POCKET_NEW" => {
            Some(LiveEvent::Popup { text: format!("{} 发出了红包", str_of(&d["sender_name"])) })
        }
        "GOTO_BUY_FLOW" | "HOT_BUY_NUM" | "SELL_GOODS_UPDATE" => {
            Some(LiveEvent::Popup { text: plain(&first_text(d, &["text", "title", "content"])) })
        }
        "WATCHED_CHANGE" => Some(LiveEvent::Online { count: u64_of(&d["num"]) }),
        "LIVE" => Some(LiveEvent::Status { live: true }),
        "PREPARING" => Some(LiveEvent::Status { live: false }),
        _ => None,
    }
}

fn first_text(d: &Value, keys: &[&str]) -> String {
    keys.iter().map(|k| str_of(&d[*k])).find(|s| !s.is_empty()).unwrap_or_default()
}

/// Drops the `<%name%>` / `<%%>` markers the channel puts around highlighted words.
fn plain(s: &str) -> String {
    s.replace("<%", "").replace("%>", "")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// 2026-11-02: the shapes of the live home page's 推荐 list and the area list.
    #[test]
    fn reads_rooms_and_areas() {
        let d = json!([{"roomid": 7734200, "title": "S16", "uname": "赛事", "face": "//i0.hdslb.com/f.jpg",
            "cover": "https://i0.hdslb.com/c.jpg", "online": 1485256, "area_v2_name": "英雄联盟", "uid": 50329118},
            {"roomid": 0}]);
        let rooms = rooms_in(&d);
        assert_eq!(rooms.len(), 1);
        let r = &rooms[0];
        assert_eq!((r.room_id, r.online, r.area.as_str(), r.live), (7734200, 1485256, "英雄联盟", true));
        assert_eq!(r.face, "https://i0.hdslb.com/f.jpg");

        let a =
            areas_of(&json!([{"id": 2, "name": "网游", "list": [{"id": "86", "parent_id": "2", "name": "英雄联盟"}]}]));
        assert_eq!(a[0].children[0], LiveSubArea { id: 86, parent_id: 2, name: "英雄联盟".into() });
    }

    #[test]
    fn a_danmaku_turned_down_says_why() {
        assert_eq!(refusal_of(""), None);
        assert_eq!(refusal_of("0"), None);
        assert_eq!(refusal_of("f").as_deref(), Some("弹幕被系统屏蔽"));
        assert_eq!(refusal_of("msg in 1s").as_deref(), Some("msg in 1s"));
    }

    /// 2026-11-06: `user/following` leaves cover, viewers and area empty.
    #[test]
    fn reads_followed_rooms() {
        let room = followed_room(&json!({"roomid": 7, "uname": "猫", "title": "夜", "face": "//i0.hdslb.com/f.jpg",
            "live_status": 1, "room_cover": "", "text_small": "", "area_name_v2": ""}));
        assert_eq!((room.room_id, room.online, room.live), (7, 0, true));
        assert_eq!(room.cover, "https://i0.hdslb.com/f.jpg");
        assert!(!followed_room(&json!({"roomid": 8, "live_status": 0})).live);
    }

    #[test]
    fn reads_a_room_header() {
        let d = json!({"room_info": {"room_id": 7734200, "short_id": 6, "uid": 50329118, "title": "总决赛",
            "cover": "https://i0.hdslb.com/c.jpg", "online": 10, "live_status": 0, "area_name": "游戏赛事"},
            "anchor_info": {"base_info": {"uname": "哔哩哔哩英雄联盟赛事", "face": "https://i2.hdslb.com/f.jpg"}},
            "watched_show": {"num": 1485256}});
        let r = room_of_info(&d);
        assert_eq!((r.room_id, r.uname.as_str(), r.online, r.live), (7734200, "哔哩哔哩英雄联盟赛事", 1485256, false));
    }

    /// 2026-11-02: `getRoomPlayInfo` with both protocols and codecs.
    #[test]
    fn picks_avc_streams_hls_first() {
        let codec = |name: &str, path: &str| {
            json!({"codec_name": name, "current_qn": 10000, "accept_qn": [10000, 400, 150], "base_url": path,
                "url_info": [{"host": "https://a.bilivideo.com", "extra": "?x=1"}, {"host": "https://b.bilivideo.com", "extra": "?x=2"}]})
        };
        let d = json!({"live_status": 1, "playurl_info": {"playurl": {
        "g_qn_desc": [{"qn": 10000, "desc": "原画"}, {"qn": 400, "desc": "蓝光"}, {"qn": 150, "desc": "高清"}],
        "stream": [
            {"protocol_name": "http_stream", "format": [{"format_name": "flv", "codec": [codec("avc", "/live.flv")]}]},
            {"protocol_name": "http_hls", "format": [
                {"format_name": "ts", "codec": [codec("avc", "/live.m3u8")]},
                {"format_name": "fmp4", "codec": [codec("hevc", "/h.m3u8"), codec("avc", "/index.m3u8")]}]}
        ]}}});
        let p = play_of(&d);
        assert!(p.live);
        assert_eq!(p.qn, 10000);
        assert_eq!(p.qualities, vec![(10000, "原画".into()), (400, "蓝光".into()), (150, "高清".into())]);
        let urls: Vec<&str> = p.streams.iter().map(|s| s.url.as_str()).collect();
        assert_eq!(urls[0], "https://a.bilivideo.com/index.m3u8?x=1");
        assert_eq!(urls[2], "https://a.bilivideo.com/live.m3u8?x=1");
        assert_eq!(p.streams[4].protocol, LiveProtocol::Flv);
        assert_eq!(p.streams.len(), 6);
        assert!(!urls.iter().any(|u| u.contains("/h.m3u8")));
    }

    #[test]
    fn a_room_off_air_has_no_streams() {
        let p = play_of(&json!({"live_status": 0, "playurl_info": null}));
        assert!(!p.live && p.streams.is_empty());
    }

    #[test]
    fn danmu_hosts_become_wss_urls() {
        let s = danmu_server_of(
            &json!({"token": "t", "host_list": [{"host": "zj-cn-live-comet.chat.bilibili.com", "wss_port": 2245}]}),
        );
        assert_eq!(s.token, "t");
        assert_eq!(
            s.urls,
            vec!["wss://zj-cn-live-comet.chat.bilibili.com:2245/sub".to_string(), FALLBACK_HOST.to_string()]
        );
    }

    #[test]
    fn sorts_channel_messages() {
        let dm = json!({"cmd": "DANMU_MSG", "info": [[0, 1, 25, 16777215, 0, 0, 0, "", 0, 0, 0, "", 0], "BLG！！", [1, "观众"]]});
        assert_eq!(
            event_of(&dm),
            Some(LiveEvent::Danmaku { uname: "观众".into(), text: "BLG！！".into(), color: 0xFF_FFFF })
        );
        let lottery = json!({"cmd": "DANMU_MSG:4:0:2:2:2:0", "info": [[0, 1, 25, 16777215, 0, 0, 0, "", 0, 2, 0, "", 0], "老板大气", [1, "观众"]]});
        assert!(matches!(event_of(&lottery), Some(LiveEvent::Popup { .. })));
        let sticker = json!({"cmd": "DANMU_MSG", "info": [[0, 1, 25, 16777215, 0, 0, 0, "", 0, 0, 0, "", 1], "[dog]", [1, "观众"]]});
        assert!(matches!(event_of(&sticker), Some(LiveEvent::Effect { .. })));
        let gift =
            json!({"cmd": "SEND_GIFT", "data": {"uname": "用户X", "action": "投喂", "giftName": "辣条", "num": 3}});
        assert_eq!(event_of(&gift), Some(LiveEvent::Gift { text: "用户X 投喂 辣条 ×3".into() }));
        let entry = json!({"cmd": "ENTRY_EFFECT", "data": {"copy_writing": "欢迎舰长 <%某人%> 进入直播间"}});
        assert_eq!(event_of(&entry), Some(LiveEvent::Effect { text: "欢迎舰长 某人 进入直播间".into() }));
        let lot = json!({"cmd": "ANCHOR_LOT_START", "data": {"award_name": "周边", "require_text": "关注主播"}});
        assert!(matches!(event_of(&lot), Some(LiveEvent::Popup { .. })));
        assert_eq!(
            event_of(&json!({"cmd": "WATCHED_CHANGE", "data": {"num": 42}})),
            Some(LiveEvent::Online { count: 42 })
        );
        assert_eq!(event_of(&json!({"cmd": "PREPARING"})), Some(LiveEvent::Status { live: false }));
        assert_eq!(event_of(&json!({"cmd": "ONLINE_RANK_V2"})), None);
    }
}
