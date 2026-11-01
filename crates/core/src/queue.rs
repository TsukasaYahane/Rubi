//! 听视频: what is being heard and how 上一首 / 下一首 move through the play list. Pure state;
//! the list and its mode are passed in, so the rules can be tested without a store.

use rubi_model::VideoCard;

use crate::playlist::{PlaylistMode, next_in, previous_in};

/// How far back 上一首 can go in shuffle.
const HEARD_KEPT: usize = 200;

#[derive(Default)]
pub(crate) struct ListenQueue {
    current: Option<VideoCard>,
    /// What was heard before `current`, oldest first.
    heard: Vec<VideoCard>,
}

impl ListenQueue {
    pub(crate) fn current(&self) -> Option<VideoCard> {
        self.current.clone()
    }

    /// Starts over from `card`.
    pub(crate) fn start(&mut self, card: VideoCard) -> VideoCard {
        self.heard.clear();
        self.current = Some(card.clone());
        card
    }

    /// Moves to `card` picked from the list.
    pub(crate) fn jump(&mut self, card: VideoCard) -> VideoCard {
        self.go(card.clone());
        card
    }

    /// After the current item. `by_user` (下一首) always moves on: in 单曲循环 to the next in
    /// order, and at the end of a sequential list back to the first. An item ending by itself
    /// follows the mode and may stop (`None`).
    pub(crate) fn next(
        &mut self,
        items: &[VideoCard],
        mode: PlaylistMode,
        by_user: bool,
        random: impl Fn(usize) -> usize,
    ) -> Option<VideoCard> {
        let now = self.current.clone()?;
        let mode = if by_user && mode == PlaylistMode::LoopOne { PlaylistMode::LoopAll } else { mode };
        let following = next_in(items, &now.bvid, mode, random).or_else(|| if by_user { items.first() } else { None });
        match following {
            Some(card) => {
                let card = card.clone();
                if card.bvid == now.bvid {
                    self.current = Some(card.clone());
                } else {
                    self.go(card.clone());
                }
                Some(card)
            }
            None => {
                self.stop();
                None
            }
        }
    }

    /// 上一首: in shuffle, back through what was heard; otherwise the item before in order.
    pub(crate) fn previous(&mut self, items: &[VideoCard], mode: PlaylistMode) -> Option<VideoCard> {
        let now = self.current.clone()?;
        let back = if mode == PlaylistMode::Shuffle && !self.heard.is_empty() {
            self.heard.pop()
        } else {
            previous_in(items, &now.bvid).cloned()
        }?;
        self.current = Some(back.clone());
        Some(back)
    }

    pub(crate) fn stop(&mut self) {
        self.current = None;
        self.heard.clear();
    }

    fn go(&mut self, card: VideoCard) {
        if let Some(now) = self.current.replace(card) {
            self.heard.push(now);
            if self.heard.len() > HEARD_KEPT {
                self.heard.remove(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(b: &str) -> VideoCard {
        VideoCard { bvid: b.into(), ..Default::default() }
    }

    fn ids(c: Option<VideoCard>) -> Option<String> {
        c.map(|c| c.bvid)
    }

    #[test]
    fn user_next_always_moves_on() {
        let items = vec![card("a"), card("b")];
        let mut q = ListenQueue::default();
        q.start(card("a"));
        assert_eq!(ids(q.next(&items, PlaylistMode::LoopOne, true, |_| 0)), Some("b".into()));
        assert_eq!(ids(q.next(&items, PlaylistMode::Sequential, true, |_| 0)), Some("a".into()), "wraps to the first");
    }

    #[test]
    fn ending_follows_the_mode() {
        let items = vec![card("a"), card("b")];
        let mut q = ListenQueue::default();
        q.start(card("b"));
        assert_eq!(ids(q.next(&items, PlaylistMode::LoopOne, false, |_| 0)), Some("b".into()));
        assert_eq!(ids(q.next(&items, PlaylistMode::Sequential, false, |_| 0)), None, "a sequential list stops");
        assert_eq!(q.current(), None);
    }

    #[test]
    fn shuffle_goes_back_the_way_it_came() {
        let items = vec![card("a"), card("b"), card("c")];
        let mut q = ListenQueue::default();
        q.start(card("a"));
        q.next(&items, PlaylistMode::Shuffle, true, |_| 1); // → c
        q.next(&items, PlaylistMode::Shuffle, true, |_| 0); // → a
        assert_eq!(ids(q.previous(&items, PlaylistMode::Shuffle)), Some("c".into()));
        assert_eq!(ids(q.previous(&items, PlaylistMode::Shuffle)), Some("a".into()));
        assert_eq!(ids(q.previous(&items, PlaylistMode::Shuffle)), Some("c".into()), "then by list order");
    }
}
