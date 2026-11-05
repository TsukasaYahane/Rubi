//! 收藏夹: the user's favourite folders, what is in them, and managing both. Putting the
//! video on show into folders is the video page's (`Actions`).

use std::sync::Arc;

use rubi_api::FavoritesApi;
use rubi_model::{FavFolder, VideoCard};

use crate::error::{CoreError, Result};
use crate::videos::CardCache;

pub struct Favorites {
    api: Arc<dyn FavoritesApi>,
    cards: Arc<CardCache>,
}

impl Favorites {
    pub(crate) fn new(api: Arc<dyn FavoritesApi>, cards: Arc<CardCache>) -> Self {
        Self { api, cards }
    }

    /// The user's own folders.
    pub async fn folders(&self) -> Result<Vec<FavFolder>> {
        Ok(self.api.my_folders().await?)
    }

    /// Page `pn` (from 1) of a folder; also whether more follow.
    pub async fn items(&self, folder_id: u64, pn: u32) -> Result<(Vec<VideoCard>, bool)> {
        let (cards, more) = self.api.fav_items(folder_id, pn.max(1)).await?;
        self.cards.remember(&cards);
        Ok((cards, more))
    }

    /// A new folder (title trimmed, 1–20 characters); returns its id.
    pub async fn create_folder(&self, title: &str, intro: &str, private: bool) -> Result<u64> {
        let title = folder_title(title)?;
        Ok(self.api.create_folder(&title, intro.trim(), private).await?)
    }

    pub async fn edit_folder(&self, id: u64, title: &str, intro: &str, private: bool) -> Result<()> {
        let title = folder_title(title)?;
        Ok(self.api.edit_folder(id, &title, intro.trim(), private).await?)
    }

    pub async fn delete_folder(&self, id: u64) -> Result<()> {
        Ok(self.api.delete_folder(id).await?)
    }

    pub async fn folder_intro(&self, id: u64) -> Result<String> {
        Ok(self.api.folder_intro(id).await?)
    }

    /// Takes videos (by aid) out of `folder`.
    pub async fn remove_from_folder(&self, folder: u64, aids: &[u64]) -> Result<()> {
        if aids.is_empty() {
            return Ok(());
        }
        Ok(self.api.remove_from_folder(folder, aids).await?)
    }

    /// Moves videos to another folder, or copies them there when `keep`.
    pub async fn transfer(&self, from: u64, to: u64, aids: &[u64], keep: bool) -> Result<()> {
        if aids.is_empty() || from == to {
            return Ok(());
        }
        Ok(self.api.transfer(from, to, aids, keep).await?)
    }
}

/// Longest folder title B站 takes.
const MAX_FOLDER_TITLE: usize = 20;

fn folder_title(title: &str) -> Result<String> {
    let title = title.trim();
    if title.is_empty() {
        return Err(CoreError::Invalid("请输入收藏夹名称".into()));
    }
    if title.chars().count() > MAX_FOLDER_TITLE {
        return Err(CoreError::Invalid(format!("名称最多 {MAX_FOLDER_TITLE} 个字")));
    }
    Ok(title.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_titles_are_trimmed_and_bounded() {
        assert_eq!(folder_title("  我的  ").unwrap(), "我的");
        assert!(folder_title("   ").is_err());
        assert!(folder_title(&"字".repeat(21)).is_err());
        assert!(folder_title(&"字".repeat(20)).is_ok());
    }
}
