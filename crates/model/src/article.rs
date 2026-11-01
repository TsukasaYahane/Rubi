use serde::{Deserialize, Serialize};

use crate::CommentPicture;

/// One block of a 专栏 / 图文 body, in reading order. Styling (bold, colours) is dropped; the
/// reader lays text out in its own type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArticleBlock {
    Heading(String),
    Text(String),
    /// A quote or a list item: text set apart from the body.
    Quote(String),
    Pictures(Vec<CommentPicture>),
    /// A horizontal rule.
    Line,
}

/// A 专栏 article or 图文 post, ready to read.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Article {
    /// The 图文 (opus) id it is read by.
    pub id: String,
    pub title: String,
    pub author_name: String,
    pub author_mid: u64,
    pub author_face: String,
    /// Unix seconds.
    pub published_at: i64,
    /// The banner above the title; empty when there is none.
    pub cover: String,
    pub blocks: Vec<ArticleBlock>,
    /// Its comment area (`type`, `oid`); kind 0 when it has none.
    pub comment_kind: u32,
    pub comment_oid: u64,
    /// Promotion cards the rules took out of the body.
    pub hidden: usize,
    /// The paragraph each block came from, in step with `blocks`, for the rule table; emptied
    /// once the rules have run.
    #[serde(skip)]
    pub raw: Vec<serde_json::Value>,
}
