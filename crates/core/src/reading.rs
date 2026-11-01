//! Reading 专栏 articles and 图文 posts inside Rubi.

use std::sync::Arc;

use rubi_api::ArticleApi;
use rubi_filter::Target as RuleTarget;
use rubi_model::Article;

use crate::error::{CoreError, Result};
use crate::ports::RuleSource;

pub struct Reading {
    api: Arc<dyn ArticleApi>,
    rules: Arc<dyn RuleSource>,
}

impl Reading {
    pub(crate) fn new(api: Arc<dyn ArticleApi>, rules: Arc<dyn RuleSource>) -> Self {
        Self { api, rules }
    }

    /// Opens what a link points to: `…/opus/ID`, `…/read/cvN` (or `cvN`), or a bare opus id.
    /// Paragraphs the rule table blocks (promotion cards, shop links) are taken out and counted.
    pub async fn open(&self, link: &str) -> Result<Article> {
        let mut article = match target_of(link) {
            Some(Target::Opus(id)) => self.api.opus(&id).await?,
            Some(Target::Cv(cv)) => self.api.article(cv).await?,
            None => return Err(CoreError::Invalid(format!("不是专栏或图文链接：{link}"))),
        };
        let pipeline = self.rules.rules().raw_pipeline(RuleTarget::ArticleBlock);
        let raw = std::mem::take(&mut article.raw);
        let before = article.blocks.len();
        article.blocks = std::mem::take(&mut article.blocks)
            .into_iter()
            .zip(raw.iter().map(Some).chain(std::iter::repeat(None)))
            .filter(|(_, p)| p.is_none_or(|p| !pipeline.judge(p).is_blocked()))
            .map(|(b, _)| b)
            .collect();
        article.hidden = before - article.blocks.len();
        Ok(article)
    }
}

#[derive(Debug, PartialEq)]
enum Target {
    Opus(String),
    Cv(u64),
}

fn digits_after(link: &str, marker: &str) -> Option<String> {
    let rest = &link[link.find(marker)? + marker.len()..];
    let d: String = rest.chars().take_while(char::is_ascii_digit).collect();
    (!d.is_empty()).then_some(d)
}

fn target_of(link: &str) -> Option<Target> {
    let link = link.trim();
    if let Some(id) = digits_after(link, "opus/") {
        return Some(Target::Opus(id));
    }
    if let Some(cv) = digits_after(link, "cv").and_then(|d| d.parse().ok()) {
        return Some(Target::Cv(cv));
    }
    (!link.is_empty() && link.chars().all(|c| c.is_ascii_digit())).then(|| Target::Opus(link.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_point_to_opus_or_cv() {
        assert_eq!(target_of("https://www.bilibili.com/opus/123?spm=x"), Some(Target::Opus("123".into())));
        assert_eq!(target_of("//www.bilibili.com/read/cv456"), Some(Target::Cv(456)));
        assert_eq!(target_of("cv7"), Some(Target::Cv(7)));
        assert_eq!(target_of("789"), Some(Target::Opus("789".into())));
        assert_eq!(target_of("https://www.bilibili.com/video/BV1x"), None);
    }
}
