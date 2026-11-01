//! The default emote set (`[doge]` → image), fetched once per process.

use std::sync::Arc;

use rubi_api::{DEFAULT_PACKAGES, Emote, EmoteApi};
use tokio::sync::Mutex;

use crate::error::Result;

pub struct Emotes {
    api: Arc<dyn EmoteApi>,
    cached: Mutex<Option<Arc<Vec<Emote>>>>,
}

impl Emotes {
    pub(crate) fn new(api: Arc<dyn EmoteApi>) -> Self {
        Self { api, cached: Mutex::new(None) }
    }

    pub async fn all(&self) -> Result<Arc<Vec<Emote>>> {
        let mut cached = self.cached.lock().await;
        if let Some(e) = cached.as_ref() {
            return Ok(e.clone());
        }
        let list = Arc::new(self.api.packages(DEFAULT_PACKAGES).await?);
        *cached = Some(list.clone());
        Ok(list)
    }
}
