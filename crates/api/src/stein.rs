//! 互动视频 (interactive videos): the story graph a branching video plays through.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::error::Result;
use crate::http::HttpClient;
use crate::params;
use crate::parse::{str_of, u64_of};

const PLAYER: &str = "https://api.bilibili.com/x/player/v2";
const EDGE: &str = "https://api.bilibili.com/x/stein/edgeinfo_v2";

/// One node of the story: the part being played and the question asked near its end.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SteinNode {
    pub edge_id: u64,
    pub title: String,
    /// An ending: no question follows.
    pub is_leaf: bool,
    pub questions: Vec<SteinQuestion>,
    /// Hidden variables with their values as the server sees them.
    pub vars: Vec<SteinVar>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SteinQuestion {
    /// 0: no options shown (jump on by itself), 1: options at the bottom, 2: options placed on
    /// the picture. Others are unknown and treated like 1.
    pub kind: u8,
    /// Shown this many ms before the part ends.
    pub before_end_ms: u32,
    /// Answer time in ms; `None` means no limit.
    pub time_limit_ms: Option<u32>,
    /// Pause at the end until a choice is made.
    pub pause: bool,
    pub title: String,
    pub choices: Vec<SteinChoice>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SteinChoice {
    /// The node this choice leads to.
    pub edge_id: u64,
    /// The part (cid) that node plays.
    pub cid: u64,
    pub option: String,
    /// Shown only when this expression over the hidden variables holds; empty: always.
    pub condition: String,
    /// Variable assignments run when chosen (`$a=$a+1;$b=0`).
    pub action: String,
    pub is_default: bool,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SteinVar {
    /// The name expressions use (`$xxx`).
    pub id: String,
    pub name: String,
    pub value: f64,
    pub shown: bool,
}

#[async_trait]
pub trait SteinApi: Send + Sync {
    /// The story graph version of an interactive video's part; `None` for ordinary videos.
    async fn graph_version(&self, bvid: &str, cid: u64) -> Result<Option<u64>>;
    /// A node of the graph; `edge_id` `None` is the start.
    async fn node(&self, bvid: &str, graph_version: u64, edge_id: Option<u64>) -> Result<SteinNode>;
}

pub struct WebStein {
    http: Arc<HttpClient>,
}

impl WebStein {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

pub(crate) fn node_of(d: &Value) -> SteinNode {
    let questions = d["edges"]["questions"]
        .as_array()
        .map(|qs| {
            qs.iter()
                .map(|q| SteinQuestion {
                    kind: u64_of(&q["type"]) as u8,
                    before_end_ms: u64_of(&q["start_time_r"]) as u32,
                    time_limit_ms: q["duration"].as_i64().filter(|&d| d > 0).map(|d| d as u32),
                    pause: u64_of(&q["pause_video"]) == 1,
                    title: str_of(&q["title"]),
                    choices: q["choices"]
                        .as_array()
                        .map(|cs| {
                            cs.iter()
                                .map(|c| SteinChoice {
                                    edge_id: u64_of(&c["id"]),
                                    cid: u64_of(&c["cid"]),
                                    option: str_of(&c["option"]),
                                    condition: str_of(&c["condition"]),
                                    action: str_of(&c["native_action"]),
                                    is_default: u64_of(&c["is_default"]) == 1,
                                    is_hidden: u64_of(&c["is_hidden"]) == 1,
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();
    let vars = d["hidden_vars"]
        .as_array()
        .map(|vs| {
            vs.iter()
                .map(|v| SteinVar {
                    id: str_of(&v["id_v2"]),
                    name: str_of(&v["name"]),
                    value: v["value"].as_f64().unwrap_or(0.0),
                    shown: u64_of(&v["is_show"]) == 1,
                })
                .collect()
        })
        .unwrap_or_default();
    SteinNode {
        edge_id: u64_of(&d["edge_id"]),
        title: str_of(&d["title"]),
        is_leaf: u64_of(&d["is_leaf"]) == 1,
        questions,
        vars,
    }
}

#[async_trait]
impl SteinApi for WebStein {
    async fn graph_version(&self, bvid: &str, cid: u64) -> Result<Option<u64>> {
        let d = self.http.get_json(PLAYER, params!["bvid" => bvid, "cid" => cid], false).await?;
        Ok(Some(u64_of(&d["interaction"]["graph_version"])).filter(|&g| g > 0))
    }

    async fn node(&self, bvid: &str, graph_version: u64, edge_id: Option<u64>) -> Result<SteinNode> {
        let mut p = params!["bvid" => bvid, "graph_version" => graph_version];
        if let Some(e) = edge_id {
            p.push(("edge_id".into(), e.to_string()));
        }
        let d = self.http.get_json(EDGE, p, false).await?;
        Ok(node_of(&d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Shape from a 2026-10-31 capture of BV1vb4y1r7cg's start node.
    #[test]
    fn parses_a_node() {
        let d = json!({"title": "选择角色", "edge_id": 1, "is_leaf": 0,
            "edges": {"questions": [{"id": 0, "type": 1, "start_time_r": 300, "duration": -1, "pause_video": 1, "title": "",
                "choices": [{"id": 19458529, "cid": 381841243, "option": "A 华强", "condition": "", "native_action": "", "is_default": 1},
                            {"id": 19458530, "cid": 381841253, "option": "B 瓜摊老板", "condition": "$x>=1", "native_action": "$x=$x+1"}]}]},
            "hidden_vars": [{"value": 0, "id_v2": "$x", "name": "好感", "is_show": 1}]});
        let n = node_of(&d);
        assert_eq!((n.edge_id, n.is_leaf, n.title.as_str()), (1, false, "选择角色"));
        let q = &n.questions[0];
        assert_eq!((q.kind, q.before_end_ms, q.time_limit_ms, q.pause), (1, 300, None, true));
        assert_eq!(q.choices[1].condition, "$x>=1");
        assert!(q.choices[0].is_default);
        assert_eq!(n.vars[0].id, "$x");
    }
}
