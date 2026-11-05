//! A room's message channel: a websocket carrying framed packets. Each packet is a 16-byte
//! header (length, header length, protocol version, operation, sequence) and a body; with
//! version 2 the body is zlib and holds more packets. Messages are JSON with a `cmd`.

use std::io::Read;
use std::time::Duration;

use async_trait::async_trait;

use flate2::read::ZlibDecoder;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio::time::{Instant, Interval, interval_at};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::error::{ApiError, Result};
use crate::http::UA;

const HEADER_LEN: usize = 16;
const OP_HEARTBEAT: u32 = 2;
const OP_MESSAGE: u32 = 5;
const OP_AUTH: u32 = 7;
/// Body versions: 0 plain JSON, 1 a number, 2 zlib, 3 brotli (never asked for).
const VER_ZLIB: u16 = 2;
const HEARTBEAT_EVERY: Duration = Duration::from_secs(30);
/// No more than this many nested zlib layers (a real server sends one).
const MAX_DEPTH: u8 = 2;
/// A frame inflates to no more than this (a real one is a few KB); a bigger one is dropped.
const MAX_INFLATED: u64 = 4 * 1024 * 1024;

/// How to join a room's channel.
pub struct ChannelJoin<'a> {
    pub room_id: u64,
    /// 0 for guests.
    pub uid: u64,
    pub buvid: &'a str,
    pub token: &'a str,
}

/// Opens room channels. The service holds this, never the websocket type, so the channel
/// can be faked in a test.
#[async_trait]
pub trait ChannelOpener: Send + Sync {
    /// Connects to the first address that answers and joins the room.
    async fn open(&self, urls: &[String], join: &ChannelJoin<'_>) -> Result<Box<dyn ChannelStream>>;
}

/// An open room channel.
#[async_trait]
pub trait ChannelStream: Send {
    /// The messages of the next packet; `None` once the channel has closed. Reading also keeps
    /// the channel alive.
    async fn next(&mut self) -> Result<Option<Vec<Value>>>;
    async fn close(self: Box<Self>);
}

/// Room channels over B站's websocket.
pub struct WebChannels;

#[async_trait]
impl ChannelOpener for WebChannels {
    async fn open(&self, urls: &[String], join: &ChannelJoin<'_>) -> Result<Box<dyn ChannelStream>> {
        Ok(Box::new(LiveSocket::open(urls, join).await?))
    }
}

#[async_trait]
impl ChannelStream for LiveSocket {
    async fn next(&mut self) -> Result<Option<Vec<Value>>> {
        LiveSocket::next(self).await
    }

    async fn close(self: Box<Self>) {
        LiveSocket::close(*self).await
    }
}

/// An open channel; read it with [`next`](Self::next), which also keeps it alive.
struct LiveSocket {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    heartbeat: Interval,
}

impl LiveSocket {
    /// Connects to the first address that answers and joins the room.
    async fn open(urls: &[String], join: &ChannelJoin<'_>) -> Result<Self> {
        let mut last = ApiError::Decode("no channel address".into());
        for url in urls {
            match Self::open_one(url, join).await {
                Ok(s) => return Ok(s),
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    async fn open_one(url: &str, join: &ChannelJoin<'_>) -> Result<Self> {
        let mut req = url.into_client_request().map_err(ws_error)?;
        let headers = req.headers_mut();
        headers.insert("User-Agent", UA.parse().map_err(|_| ApiError::Decode("bad UA".into()))?);
        headers
            .insert("Origin", "https://live.bilibili.com".parse().map_err(|_| ApiError::Decode("bad origin".into()))?);
        let (mut ws, _) = connect_async(req).await.map_err(ws_error)?;
        let auth = json!({
            "uid": join.uid, "roomid": join.room_id, "protover": VER_ZLIB, "buvid": join.buvid,
            "platform": "web", "type": 2, "key": join.token,
        });
        ws.send(Message::Binary(packet(OP_AUTH, auth.to_string().as_bytes()).into())).await.map_err(ws_error)?;
        // The first heartbeat goes out right after joining, then every 30 s.
        let heartbeat = interval_at(Instant::now(), HEARTBEAT_EVERY);
        Ok(Self { ws, heartbeat })
    }

    /// The next messages from the channel, sending heartbeats while it waits. `None` once the
    /// server closes the channel.
    async fn next(&mut self) -> Result<Option<Vec<Value>>> {
        loop {
            tokio::select! {
                _ = self.heartbeat.tick() => {
                    self.ws.send(Message::Binary(packet(OP_HEARTBEAT, b"[object Object]").into())).await.map_err(ws_error)?;
                }
                frame = self.ws.next() => {
                    let Some(frame) = frame else { return Ok(None) };
                    match frame.map_err(ws_error)? {
                        Message::Binary(bytes) => {
                            let found = messages_in(&bytes);
                            if !found.is_empty() {
                                return Ok(Some(found));
                            }
                        }
                        Message::Close(_) => return Ok(None),
                        _ => {}
                    }
                }
            }
        }
    }

    async fn close(mut self) {
        let _ = self.ws.close(None).await;
    }
}

fn ws_error(e: impl std::fmt::Display) -> ApiError {
    ApiError::Decode(format!("live channel: {e}"))
}

/// One packet with a body.
pub(crate) fn packet(op: u32, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + body.len());
    out.extend_from_slice(&((HEADER_LEN + body.len()) as u32).to_be_bytes());
    out.extend_from_slice(&(HEADER_LEN as u16).to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&op.to_be_bytes());
    out.extend_from_slice(&1u32.to_be_bytes());
    out.extend_from_slice(body);
    out
}

/// Every JSON message in a frame. Heartbeat and join replies carry none; broken packets are
/// skipped rather than failing the channel.
pub(crate) fn messages_in(frame: &[u8]) -> Vec<Value> {
    let mut out = Vec::new();
    collect(frame, 0, &mut out);
    out
}

fn collect(mut buf: &[u8], depth: u8, out: &mut Vec<Value>) {
    while buf.len() >= HEADER_LEN {
        let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        let header = u16::from_be_bytes([buf[4], buf[5]]) as usize;
        let ver = u16::from_be_bytes([buf[6], buf[7]]);
        let op = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        if len < header || header < HEADER_LEN || len > buf.len() {
            return;
        }
        let body = &buf[header..len];
        match (op, ver) {
            (OP_MESSAGE, VER_ZLIB) if depth < MAX_DEPTH => {
                let mut inner = Vec::new();
                let mut decoder = ZlibDecoder::new(body).take(MAX_INFLATED + 1);
                if decoder.read_to_end(&mut inner).is_ok() && inner.len() as u64 <= MAX_INFLATED {
                    collect(&inner, depth + 1, out);
                }
            }
            (OP_MESSAGE, 0) => {
                if let Ok(v) = serde_json::from_slice::<Value>(body) {
                    out.push(v);
                }
            }
            // Heartbeat replies (a viewer number) and the join reply carry nothing to show.
            _ => {}
        }
        buf = &buf[len..];
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::Compression;
    use flate2::write::ZlibEncoder;

    use super::*;

    fn raw(op: u32, ver: u16, body: &[u8]) -> Vec<u8> {
        let mut p = packet(op, body);
        p[6..8].copy_from_slice(&ver.to_be_bytes());
        p
    }

    #[test]
    fn frames_a_packet() {
        let p = packet(OP_AUTH, b"{}");
        assert_eq!(p.len(), 18);
        assert_eq!(&p[..4], &18u32.to_be_bytes());
        assert_eq!(&p[8..12], &OP_AUTH.to_be_bytes());
    }

    #[test]
    fn reads_plain_and_zlib_messages() {
        let a = raw(OP_MESSAGE, 0, br#"{"cmd":"A"}"#);
        let b = raw(OP_MESSAGE, 0, br#"{"cmd":"B"}"#);
        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        z.write_all(&[a.clone(), b].concat()).unwrap();
        let zipped = raw(OP_MESSAGE, VER_ZLIB, &z.finish().unwrap());
        let frame = [raw(3, 1, &7u32.to_be_bytes()), zipped, a].concat();
        let cmds: Vec<String> = messages_in(&frame).iter().map(|v| v["cmd"].as_str().unwrap().to_owned()).collect();
        assert_eq!(cmds, ["A", "B", "A"]);
    }

    #[test]
    fn stops_at_a_broken_packet() {
        let mut p = raw(OP_MESSAGE, 0, br#"{"cmd":"A"}"#);
        p[3] = 200; // claims more than there is
        assert!(messages_in(&p).is_empty());
        assert!(messages_in(&[0; 5]).is_empty());
    }
}
