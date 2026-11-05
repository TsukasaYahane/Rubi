//! Like, coin, favourite, watch later, follow.

use crate::{ActionsService, FfiFavFolder, FfiTriple, FfiVideoRelation, Res};

#[uniffi::export(async_runtime = "tokio")]
impl ActionsService {
    pub async fn like(&self, bvid: String, on: bool) -> Res<()> {
        Ok(self.actions.like(&bvid, on).await?)
    }

    /// Whether the user has liked / coined / favourited the video and follows its UP.
    pub async fn relation(&self, bvid: String) -> Res<FfiVideoRelation> {
        Ok(self.actions.relation(&bvid).await?.into())
    }

    /// `count` is 1 or 2; `also_like` likes the video in the same request.
    pub async fn coin(&self, bvid: String, count: u32, also_like: bool) -> Res<()> {
        Ok(self.actions.coin(&bvid, count, also_like).await?)
    }

    /// 一键三连: like, coins (none if already coined) and the default favourite folder.
    pub async fn triple(&self, bvid: String) -> Res<FfiTriple> {
        Ok(self.actions.triple(&bvid).await?.into())
    }

    /// The user's favourite folders, each marked if it already holds the video.
    pub async fn fav_folders(&self, bvid: String) -> Res<Vec<FfiFavFolder>> {
        Ok(self.actions.fav_folders(&bvid).await?.into_iter().map(Into::into).collect())
    }

    /// Adds the video to the `add` folders and removes it from the `remove` ones.
    pub async fn set_favorite(&self, bvid: String, add: Vec<u64>, remove: Vec<u64>) -> Res<()> {
        Ok(self.actions.favorite(&bvid, &add, &remove).await?)
    }

    pub async fn watch_later(&self, bvid: String) -> Res<()> {
        Ok(self.actions.watch_later(&bvid).await?)
    }

    pub async fn follow_up(&self, bvid: String, on: bool) -> Res<()> {
        Ok(self.actions.follow_up(&bvid, on).await?)
    }

    /// Follow / unfollow from the UP's page.
    pub async fn follow_mid(&self, mid: u64, name: String, on: bool) -> Res<()> {
        Ok(self.actions.follow_mid(mid, &name, on).await?)
    }
}
