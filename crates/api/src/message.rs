//! 消息中心 (replies / @ / likes) and private chat. All endpoints need a login.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use rubi_model::{ChatMessage, ChatSession, Notice, NoticeKind, UnreadCounts};
use serde_json::Value;

use crate::error::{ApiError, Result};
use crate::http::HttpClient;
use crate::params;
use crate::parse::{str_of, u64_of};

const UNREAD: &str = "https://api.bilibili.com/x/msgfeed/unread";
const FEED_REPLY: &str = "https://api.bilibili.com/x/msgfeed/reply";
const FEED_AT: &str = "https://api.bilibili.com/x/msgfeed/at";
const FEED_LIKE: &str = "https://api.bilibili.com/x/msgfeed/like";
const CHAT_UNREAD: &str = "https://api.vc.bilibili.com/session_svr/v1/session_svr/single_unread";
const SESSIONS: &str = "https://api.vc.bilibili.com/session_svr/v1/session_svr/get_sessions";
const USER_CARDS: &str = "https://api.vc.bilibili.com/account/v1/user/cards";
const MESSAGES: &str = "https://api.vc.bilibili.com/svr_sync/v1/svr_sync/fetch_session_msgs";
const SEND: &str = "https://api.vc.bilibili.com/web_im/v1/web_im/send_msg";
const ACK: &str = "https://api.vc.bilibili.com/session_svr/v1/session_svr/update_ack";

/// Opaque pagination cursor (`id:time`).
pub type NoticeCursor = String;

#[async_trait]
pub trait NoticeApi: Send + Sync {
    async fn unread(&self) -> Result<UnreadCounts>;
    async fn notices(&self, kind: NoticeKind, cursor: Option<&str>) -> Result<(Vec<Notice>, Option<NoticeCursor>)>;
}

#[async_trait]
pub trait ChatApi: Send + Sync {
    /// Sessions with names filled in, unfiltered.
    async fn sessions(&self) -> Result<Vec<ChatSession>>;
    /// Messages with `talker`, oldest first.
    async fn messages(&self, talker_mid: u64, size: u32) -> Result<Vec<ChatMessage>>;
    async fn send_text(&self, talker_mid: u64, text: &str) -> Result<()>;
    async fn mark_read(&self, talker_mid: u64, ack_seqno: u64) -> Result<()>;
}

pub struct WebMessages {
    http: Arc<HttpClient>,
    dev_seq: AtomicU64,
}

impl WebMessages {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http, dev_seq: AtomicU64::new(0) }
    }

    fn my_mid(&self) -> Result<u64> {
        self.http.session().dede_user_id.and_then(|m| m.parse().ok()).ok_or(ApiError::NotLoggedIn)
    }

    /// A UUID-shaped device id, as the web client sends.
    fn dev_id(&self) -> String {
        let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)
            ^ (u128::from(self.dev_seq.fetch_add(1, Ordering::Relaxed)) << 64);
        let h = format!("{n:032X}");
        format!("{}-{}-{}-{}-{}", &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32])
    }
}

/// Text of a private message `content` (a JSON string) by `msg_type`.
pub(crate) fn message_text(msg_type: u64, content: &str) -> String {
    let parsed: Value = serde_json::from_str(content).unwrap_or(Value::Null);
    match msg_type {
        1 => parsed["content"].as_str().map(str::to_owned).unwrap_or_else(|| content.to_owned()),
        2 => "[图片]".into(),
        5 => "[已撤回]".into(),
        _ => parsed["title"].as_str().map(|t| format!("[{t}]")).unwrap_or_else(|| "[消息]".into()),
    }
}

pub(crate) fn notice_of(kind: NoticeKind, v: &Value) -> Notice {
    let item = &v["item"];
    match kind {
        NoticeKind::Reply => {
            let mine = str_of(&item["target_reply_content"]);
            let title = str_of(&item["title"]);
            Notice {
                id: u64_of(&v["id"]),
                from: str_of(&v["user"]["nickname"]),
                from_mid: u64_of(&v["user"]["mid"]),
                text: str_of(&item["source_content"]),
                context: if mine.is_empty() { title } else { format!("回复我的评论「{mine}」· {title}") },
                uri: str_of(&item["uri"]),
                at: u64_of(&v["reply_time"]) as i64,
            }
        }
        NoticeKind::At => Notice {
            id: u64_of(&v["id"]),
            from: str_of(&v["user"]["nickname"]),
            from_mid: u64_of(&v["user"]["mid"]),
            text: str_of(&item["source_content"]),
            context: str_of(&item["title"]),
            uri: str_of(&item["uri"]),
            at: u64_of(&v["at_time"]) as i64,
        },
        NoticeKind::Like => {
            let users = v["users"].as_array().cloned().unwrap_or_default();
            let first = users.first().map(|u| str_of(&u["nickname"])).unwrap_or_default();
            let count = u64_of(&v["counts"]).max(users.len() as u64);
            Notice {
                id: u64_of(&v["id"]),
                from: if count > 1 { format!("{first} 等 {count} 人") } else { first },
                from_mid: users.first().map(|u| u64_of(&u["mid"])).unwrap_or(0),
                text: str_of(&item["title"]),
                context: str_of(&item["desc"]),
                uri: str_of(&item["uri"]),
                at: u64_of(&v["like_time"]) as i64,
            }
        }
    }
}

#[async_trait]
impl NoticeApi for WebMessages {
    async fn unread(&self) -> Result<UnreadCounts> {
        let d = self.http.get_json(UNREAD, params!["build" => 0, "mobi_app" => "web"], false).await?;
        let c = self
            .http
            .get_json(
                CHAT_UNREAD,
                params!["unread_type" => 0, "show_dustbin" => 1, "build" => 0, "mobi_app" => "web"],
                false,
            )
            .await?;
        Ok(UnreadCounts {
            reply: u64_of(&d["reply"]) as u32,
            at: u64_of(&d["at"]) as u32,
            like: u64_of(&d["like"]) as u32,
            chat: (u64_of(&c["follow_unread"]) + u64_of(&c["unfollow_unread"])) as u32,
        })
    }

    async fn notices(&self, kind: NoticeKind, cursor: Option<&str>) -> Result<(Vec<Notice>, Option<NoticeCursor>)> {
        let (url, time_key) = match kind {
            NoticeKind::Reply => (FEED_REPLY, "reply_time"),
            NoticeKind::At => (FEED_AT, "at_time"),
            NoticeKind::Like => (FEED_LIKE, "like_time"),
        };
        let mut p = params!["platform" => "web", "build" => 0, "mobi_app" => "web"];
        if let Some((id, time)) = cursor.and_then(|c| c.split_once(':')) {
            p.push(("id".into(), id.into()));
            p.push((time_key.into(), time.into()));
        }
        let d = self.http.get_json(url, p, false).await?;
        // Likes nest their list under `total`.
        let root = if kind == NoticeKind::Like { &d["total"] } else { &d };
        let items =
            root["items"].as_array().map(|a| a.iter().map(|v| notice_of(kind, v)).collect()).unwrap_or_default();
        let c = &root["cursor"];
        let next =
            (!c["is_end"].as_bool().unwrap_or(true)).then(|| format!("{}:{}", u64_of(&c["id"]), u64_of(&c["time"])));
        Ok((items, next))
    }
}

#[async_trait]
impl ChatApi for WebMessages {
    async fn sessions(&self) -> Result<Vec<ChatSession>> {
        let me = self.my_mid()?;
        let p = params!["session_type" => 1, "group_fold" => 1, "unfollow_fold" => 0, "sort_rule" => 2, "build" => 0, "mobi_app" => "web"];
        let d = self.http.get_json(SESSIONS, p, false).await?;
        let mut sessions: Vec<ChatSession> = d["session_list"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|s| {
                        let last = &s["last_msg"];
                        ChatSession {
                            talker_mid: u64_of(&s["talker_id"]),
                            name: String::new(),
                            unread: u64_of(&s["unread_count"]) as u32,
                            last_text: message_text(u64_of(&last["msg_type"]), &str_of(&last["content"])),
                            last_from_me: u64_of(&last["sender_uid"]) == me,
                            last_at: u64_of(&last["timestamp"]) as i64,
                            ack_seqno: u64_of(&s["max_seqno"]),
                            raw: s.clone(),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        if !sessions.is_empty() {
            let uids = sessions.iter().map(|s| s.talker_mid.to_string()).collect::<Vec<_>>().join(",");
            let cards = self.http.get_json(USER_CARDS, params!["uids" => uids], false).await?;
            for c in cards.as_array().into_iter().flatten() {
                let mid = u64_of(&c["mid"]);
                for s in sessions.iter_mut().filter(|s| s.talker_mid == mid) {
                    s.name = str_of(&c["name"]);
                }
            }
        }
        Ok(sessions)
    }

    async fn messages(&self, talker_mid: u64, size: u32) -> Result<Vec<ChatMessage>> {
        let p =
            params!["talker_id" => talker_mid, "session_type" => 1, "size" => size, "build" => 0, "mobi_app" => "web"];
        let d = self.http.get_json(MESSAGES, p, false).await?;
        let mut msgs: Vec<ChatMessage> = d["messages"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|m| ChatMessage {
                        seqno: u64_of(&m["msg_seqno"]),
                        sender_mid: u64_of(&m["sender_uid"]),
                        text: message_text(u64_of(&m["msg_type"]), &str_of(&m["content"])),
                        at: u64_of(&m["timestamp"]) as i64,
                    })
                    .collect()
            })
            .unwrap_or_default();
        msgs.sort_by_key(|m| m.seqno);
        Ok(msgs)
    }

    async fn send_text(&self, talker_mid: u64, text: &str) -> Result<()> {
        let me = self.my_mid()?;
        let csrf = self.http.session().csrf().map(str::to_owned).ok_or(ApiError::NotLoggedIn)?;
        let form = params![
            "msg[sender_uid]" => me,
            "msg[receiver_id]" => talker_mid,
            "msg[receiver_type]" => 1,
            "msg[msg_type]" => 1,
            "msg[msg_status]" => 0,
            "msg[content]" => serde_json::json!({ "content": text }),
            "msg[timestamp]" => rubi_model::now_secs(),
            "msg[new_face_version]" => 0,
            "msg[dev_id]" => self.dev_id(),
            "csrf_token" => csrf,
        ];
        self.http.post_form(SEND, form).await.map(|_| ())
    }

    async fn mark_read(&self, talker_mid: u64, ack_seqno: u64) -> Result<()> {
        let csrf = self.http.session().csrf().map(str::to_owned).ok_or(ApiError::NotLoggedIn)?;
        let form =
            params!["talker_id" => talker_mid, "session_type" => 1, "ack_seqno" => ack_seqno, "csrf_token" => csrf];
        self.http.post_form(ACK, form).await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decodes_message_content() {
        assert_eq!(message_text(1, r#"{"content":"你好"}"#), "你好");
        assert_eq!(message_text(2, r#"{"url":"x"}"#), "[图片]");
        assert_eq!(message_text(10, r#"{"title":"活动通知"}"#), "[活动通知]");
    }

    #[test]
    fn parses_notices() {
        let reply = json!({"id": 1, "user": {"mid": 2, "nickname": "阿蓝"}, "reply_time": 100,
            "item": {"title": "视频", "source_content": "Pin 能省吗", "target_reply_content": "我的评论", "uri": "u"}});
        let n = notice_of(NoticeKind::Reply, &reply);
        assert_eq!((n.from.as_str(), n.text.as_str(), n.at), ("阿蓝", "Pin 能省吗", 100));
        assert!(n.context.contains("我的评论"));

        let like = json!({"id": 3, "users": [{"mid": 4, "nickname": "Kite"}], "counts": 12, "like_time": 5,
            "item": {"title": "评论内容", "desc": "视频"}});
        assert_eq!(notice_of(NoticeKind::Like, &like).from, "Kite 等 12 人");
    }
}
