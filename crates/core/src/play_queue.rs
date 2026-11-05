//! Playing a list on the video page: where 播放全部 starts, what follows a video that ended,
//! and the temporary list a 收藏夹 or 稍后再看 is played through as.

use std::sync::Arc;

use rubi_model::VideoCard;
use rubi_store::{KvRepo, PlaylistRepo, SqliteStore};

use crate::error::Result;
use crate::lineup::{Lineup, random_below};
use crate::playlist::{PlaylistMode, TEMP_PLAYLIST, next_in};

pub struct PlayQueue<S = SqliteStore> {
    lineup: Arc<Lineup<S>>,
}

impl<S: PlaylistRepo + KvRepo + Send + Sync> PlayQueue<S> {
    pub(crate) fn new(lineup: Arc<Lineup<S>>) -> Self {
        Self { lineup }
    }

    /// Makes `list` the one played: its next item follows the video that ends.
    pub fn play(&self, list: i64) {
        self.lineup.set_active(list);
    }

    /// Where 播放全部 / 听全部 starts; `list` becomes the one played.
    pub fn first(&self, list: i64) -> Result<Option<VideoCard>> {
        self.play(list);
        let items = self.lineup.items(list)?;
        Ok(match self.lineup.mode_of(list)? {
            PlaylistMode::Shuffle if !items.is_empty() => Some(items[random_below(items.len())].clone()),
            _ => items.into_iter().next(),
        })
    }

    /// The video to play after `current` ended on the video page; `None` stops.
    pub fn next_after(&self, current: &str) -> Result<Option<VideoCard>> {
        let list = self.lineup.active();
        let items = self.lineup.items(list)?;
        Ok(next_in(&items, current, self.lineup.mode_of(list)?, random_below).cloned())
    }

    /// The list being played: its id, name and videos.
    pub fn playing(&self) -> Result<(i64, String, Vec<VideoCard>)> {
        let list = self.lineup.active();
        Ok((list, self.lineup.name_of(list)?, self.lineup.items(list)?))
    }

    /// Plays `bvids` (a 收藏夹, 稍后再看) as a list of their own that is not kept: it replaces
    /// the last such list and goes with [Self::drop_temp]. Returns the first to play.
    pub async fn play_temp(&self, name: &str, bvids: &[String]) -> Result<Option<VideoCard>> {
        let cards = self.lineup.cards_of(bvids).await;
        self.lineup.put(TEMP_PLAYLIST, name, cards);
        self.first(TEMP_PLAYLIST)
    }

    /// Adds later pages of the list being played through.
    pub async fn extend_temp(&self, bvids: &[String]) {
        let cards = self.lineup.cards_of(bvids).await;
        self.lineup.extend(TEMP_PLAYLIST, cards);
    }

    /// The video page playing the temporary list closed: the list goes.
    pub fn drop_temp(&self) {
        self.lineup.forget(TEMP_PLAYLIST);
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use rubi_store::DEFAULT_PLAYLIST;

    use super::*;
    use crate::listening::Listening;
    use crate::playlist::{Playlist, RELATED_PLAYLIST};
    use crate::ports::VideoLookup;
    use crate::videos::CardCache;

    struct Cards;

    #[async_trait]
    impl VideoLookup for Cards {
        async fn card(&self, bvid: &str) -> Result<VideoCard> {
            Ok(VideoCard { bvid: bvid.into(), ..Default::default() })
        }
    }

    fn lineup() -> Arc<Lineup<SqliteStore>> {
        let list = Arc::new(Playlist::new(Arc::new(SqliteStore::in_memory().unwrap())));
        Arc::new(Lineup::new(list, Arc::new(Cards), Arc::new(CardCache::default())))
    }

    fn ids(cards: &[VideoCard]) -> Vec<&str> {
        cards.iter().map(|c| c.bvid.as_str()).collect()
    }

    #[tokio::test]
    async fn temp_list_plays_through_then_goes() {
        let q = PlayQueue::new(lineup());
        let bv = |s: &[&str]| s.iter().map(|b| b.to_string()).collect::<Vec<_>>();
        assert_eq!(q.play_temp("收藏夹", &bv(&["a", "b", "a"])).await.unwrap().unwrap().bvid, "a");
        q.extend_temp(&bv(&["b", "c"])).await;
        let (id, name, items) = q.playing().unwrap();
        assert_eq!((id, name.as_str(), ids(&items)), (TEMP_PLAYLIST, "收藏夹", vec!["a", "b", "c"]));
        assert_eq!(q.next_after("b").unwrap().unwrap().bvid, "c");
        q.drop_temp();
        assert_eq!(q.playing().unwrap().0, DEFAULT_PLAYLIST);
    }

    #[tokio::test]
    async fn listening_follows_related_and_shares_the_lineup() {
        let shared = lineup();
        let (q, l) = (PlayQueue::new(shared.clone()), Listening::new(shared));
        l.start_related("a").await.unwrap();
        l.append(&["b".into(), "c".into()]).await;
        assert_eq!(q.playing().unwrap().0, RELATED_PLAYLIST, "the video page follows what is heard");
        assert_eq!(l.related_left(), Some(2));
        assert_eq!(l.next(true).unwrap().unwrap().bvid, "b");
        l.stop();
        assert_eq!(q.playing().unwrap().0, DEFAULT_PLAYLIST);
        assert_eq!(l.related_left(), None);
    }
}
