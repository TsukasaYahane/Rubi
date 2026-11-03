//! What the user does to a video or an UP: like, coin, favourite, watch later, follow. Each
//! one that succeeds also feeds the recommendation profile.

use std::sync::Arc;

use rubi_api::{ActionApi, Triple};
use rubi_model::{FavFolder, FeedbackEvent, ItemRef, Signal, VideoCard, VideoRelation, now_secs};

use crate::error::Result;
use crate::ports::{FeedbackSink, VideoLookup};

pub struct Actions {
    api: Arc<dyn ActionApi>,
    videos: Arc<dyn VideoLookup>,
    feedback: Arc<dyn FeedbackSink>,
}

impl Actions {
    pub(crate) fn new(api: Arc<dyn ActionApi>, videos: Arc<dyn VideoLookup>, feedback: Arc<dyn FeedbackSink>) -> Self {
        Self { api, videos, feedback }
    }

    pub async fn like(&self, bvid: &str, on: bool) -> Result<()> {
        let card = self.videos.card(bvid).await?;
        self.api.like(bvid, on).await?;
        if on { self.signal(&card, Signal::Like) } else { Ok(()) }
    }

    /// `count` is 1 or 2; `also_like` likes in the same request.
    pub async fn coin(&self, bvid: &str, count: u32, also_like: bool) -> Result<()> {
        let card = self.videos.card(bvid).await?;
        self.api.coin(bvid, count, also_like).await?;
        self.signal(&card, Signal::Coin)
    }

    /// 一键三连; each part that succeeded feeds the profile.
    pub async fn triple(&self, bvid: &str) -> Result<Triple> {
        let card = self.videos.card(bvid).await?;
        let t = self.api.triple(bvid).await?;
        let done = [(t.liked, Signal::Like), (t.coined, Signal::Coin), (t.favorited, Signal::Favorite)];
        for (_, s) in done.into_iter().filter(|(ok, _)| *ok) {
            self.signal(&card, s)?;
        }
        Ok(t)
    }

    /// Whether the user has liked / coined / favourited the video and follows its UP.
    pub async fn relation(&self, bvid: &str) -> Result<VideoRelation> {
        Ok(self.api.relation(bvid).await?)
    }

    /// The user's favourite folders, each marked if it already holds the video.
    pub async fn fav_folders(&self, bvid: &str) -> Result<Vec<FavFolder>> {
        let card = self.videos.card(bvid).await?;
        Ok(self.api.fav_folders(card.aid).await?)
    }

    /// Adds the video to the `add` folders and removes it from the `remove` ones.
    pub async fn favorite(&self, bvid: &str, add: &[u64], remove: &[u64]) -> Result<()> {
        let card = self.videos.card(bvid).await?;
        self.api.set_fav(card.aid, add, remove).await?;
        if add.is_empty() { Ok(()) } else { self.signal(&card, Signal::Favorite) }
    }

    pub async fn watch_later(&self, bvid: &str) -> Result<()> {
        Ok(self.api.watch_later(bvid).await?)
    }

    /// Follows / unfollows the UP of a video.
    pub async fn follow_up(&self, bvid: &str, on: bool) -> Result<()> {
        let card = self.videos.card(bvid).await?;
        self.api.follow(card.owner.mid, on).await?;
        if on { self.signal(&card, Signal::Follow) } else { Ok(()) }
    }

    /// Follow / unfollow from the UP's page, where there is no video to attach.
    pub async fn follow_mid(&self, mid: u64, name: &str, on: bool) -> Result<()> {
        self.api.follow(mid, on).await?;
        if !on {
            return Ok(());
        }
        let item = ItemRef { up_mid: mid, title: name.to_owned(), ..ItemRef::of(&VideoCard::default()) };
        self.feedback.record(FeedbackEvent { signal: Signal::Follow, item: Some(item), at: now_secs() })
    }

    fn signal(&self, card: &VideoCard, signal: Signal) -> Result<()> {
        self.feedback.record(FeedbackEvent { signal, item: Some(ItemRef::of(card)), at: now_secs() })
    }
}
