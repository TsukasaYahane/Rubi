//! The message center and chats.

use rubi_model::NoticeKind;

use crate::{FfiChatMessage, FfiChatSessions, FfiNoticeKind, FfiNoticePage, FfiUnread, Res, RubiCore};

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    pub async fn unread(&self) -> Res<FfiUnread> {
        Ok(self.messages.unread().await?.into())
    }

    pub async fn notices(&self, kind: FfiNoticeKind, cursor: Option<String>) -> Res<FfiNoticePage> {
        let kind = match kind {
            FfiNoticeKind::Reply => NoticeKind::Reply,
            FfiNoticeKind::At => NoticeKind::At,
            FfiNoticeKind::Like => NoticeKind::Like,
        };
        let (items, cursor) = self.messages.notices(kind, cursor.as_deref()).await?;
        Ok(FfiNoticePage { items: items.into_iter().map(Into::into).collect(), cursor })
    }

    pub async fn chat_sessions(&self) -> Res<FfiChatSessions> {
        let (sessions, hidden) = self.messages.sessions().await?;
        Ok(FfiChatSessions { sessions: sessions.into_iter().map(Into::into).collect(), hidden: hidden as u32 })
    }

    pub async fn chat_messages(&self, talker_mid: u64) -> Res<Vec<FfiChatMessage>> {
        let msgs = self.messages.chat(talker_mid).await?;
        Ok(msgs.into_iter().map(|m| FfiChatMessage::of(m, talker_mid)).collect())
    }

    /// Sends video `bvid` to `talker_mid`: as B站's share card when the server takes it, else
    /// as its title and link. True when it went as a card.
    pub async fn share_video_to(&self, talker_mid: u64, bvid: String) -> Res<bool> {
        let card = self.videos.card(&bvid).await?;
        Ok(self.messages.share_video(talker_mid, &card).await?)
    }

    pub async fn send_message(&self, talker_mid: u64, text: String) -> Res<()> {
        Ok(self.messages.send(talker_mid, &text).await?)
    }

    pub async fn mark_read(&self, talker_mid: u64, ack_seqno: u64) -> Res<()> {
        Ok(self.messages.mark_read(talker_mid, ack_seqno).await?)
    }
}
