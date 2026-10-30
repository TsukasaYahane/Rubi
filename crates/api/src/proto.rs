//! Danmaku protobuf messages (docs/danmaku/danmaku_proto.md, danmaku_view_proto.md).
//! Field numbers are unverified against a live capture; see api-notes/danmaku.md.

#[derive(Clone, PartialEq, prost::Message)]
pub struct DanmakuElem {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(int32, tag = "2")]
    pub progress: i32,
    #[prost(int32, tag = "3")]
    pub mode: i32,
    #[prost(int32, tag = "4")]
    pub fontsize: i32,
    #[prost(uint32, tag = "5")]
    pub color: u32,
    #[prost(string, tag = "6")]
    pub mid_hash: String,
    #[prost(string, tag = "7")]
    pub content: String,
    #[prost(int64, tag = "8")]
    pub ctime: i64,
    #[prost(int32, tag = "9")]
    pub weight: i32,
    #[prost(string, tag = "10")]
    pub action: String,
    #[prost(int32, tag = "11")]
    pub pool: i32,
    #[prost(string, tag = "12")]
    pub id_str: String,
    #[prost(int32, tag = "13")]
    pub attr: i32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct DmSegMobileReply {
    #[prost(message, repeated, tag = "1")]
    pub elems: Vec<DanmakuElem>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct DmSegConfig {
    #[prost(int64, tag = "1")]
    pub page_size: i64,
    #[prost(int64, tag = "2")]
    pub total: i64,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct CommandDm {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(int64, tag = "2")]
    pub oid: i64,
    /// Verified 2026-10-30: a varint, not a string as older notes suggest.
    #[prost(int64, tag = "3")]
    pub mid: i64,
    #[prost(string, tag = "4")]
    pub command: String,
    #[prost(string, tag = "5")]
    pub content: String,
    #[prost(int32, tag = "6")]
    pub progress: i32,
    #[prost(string, tag = "7")]
    pub ctime: String,
    #[prost(string, tag = "8")]
    pub mtime: String,
    #[prost(string, tag = "9")]
    pub extra: String,
    #[prost(string, tag = "10")]
    pub id_str: String,
}

/// Only the fields Rubi reads; prost skips the rest.
#[derive(Clone, PartialEq, prost::Message)]
pub struct DmWebViewReply {
    #[prost(int32, tag = "1")]
    pub state: i32,
    #[prost(message, optional, tag = "4")]
    pub dm_sge: Option<DmSegConfig>,
    #[prost(message, repeated, tag = "9")]
    pub command_dms: Vec<CommandDm>,
}
