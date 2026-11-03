//! The message center: unread counts, notices, and chats (marketing chats filtered).

use std::sync::Arc;

use rubi_api::{ChatApi, NoticeApi, NoticeCursor};
use rubi_model::{ChatMessage, ChatSession, Notice, NoticeKind, UnreadCounts};

use crate::error::Result;
use crate::ports::RuleSource;
use crate::social::filter_sessions;

/// Chat messages loaded when a conversation opens.
const CHAT_PAGE: u32 = 30;

pub struct Messages {
    notices: Arc<dyn NoticeApi>,
    chat: Arc<dyn ChatApi>,
    rules: Arc<dyn RuleSource>,
}

impl Messages {
    pub(crate) fn new(notices: Arc<dyn NoticeApi>, chat: Arc<dyn ChatApi>, rules: Arc<dyn RuleSource>) -> Self {
        Self { notices, chat, rules }
    }

    pub async fn unread(&self) -> Result<UnreadCounts> {
        Ok(self.notices.unread().await?)
    }

    pub async fn notices(&self, kind: NoticeKind, cursor: Option<&str>) -> Result<(Vec<Notice>, Option<NoticeCursor>)> {
        Ok(self.notices.notices(kind, cursor).await?)
    }

    /// Chat sessions without marketing ones; also how many were hidden.
    pub async fn sessions(&self) -> Result<(Vec<ChatSession>, usize)> {
        let sessions = self.chat.sessions().await?;
        Ok(filter_sessions(&self.rules.rules(), sessions))
    }

    pub async fn chat(&self, talker_mid: u64) -> Result<Vec<ChatMessage>> {
        Ok(self.chat.messages(talker_mid, CHAT_PAGE).await?)
    }

    /// Sends `card` to `talker_mid`: as B站's video share card when the server takes it,
    /// otherwise as its title and link. B站 answers -400 to share cards sent from outside its
    /// own clients (seen 2026-11-03); a refused card sends nothing, so the link is no repeat.
    /// Returns whether it went as a card.
    pub async fn share_video(&self, talker_mid: u64, card: &rubi_model::VideoCard) -> Result<bool> {
        let share = rubi_api::VideoShare {
            aid: card.aid,
            bvid: card.bvid.clone(),
            title: card.title.clone(),
            cover: card.cover.clone(),
            author: card.owner.name.clone(),
        };
        match self.chat.send_video(talker_mid, &share).await {
            Ok(()) => Ok(true),
            Err(rubi_api::ApiError::Api { code: -400, .. }) => {
                self.chat.send_text(talker_mid, &share_text(card)).await?;
                Ok(false)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn send(&self, talker_mid: u64, text: &str) -> Result<()> {
        Ok(self.chat.send_text(talker_mid, text).await?)
    }

    pub async fn mark_read(&self, talker_mid: u64, ack_seqno: u64) -> Result<()> {
        Ok(self.chat.mark_read(talker_mid, ack_seqno).await?)
    }
}

/// A video sent as text: its title and link (B站 shows the link as the video).
fn share_text(card: &rubi_model::VideoCard) -> String {
    format!("【{}】 https://www.bilibili.com/video/{}", card.title, card.bvid)
}
