//! 专栏 and 图文, read inside Rubi.

use rubi_model::ArticleBlock;

use crate::{FfiArticle, FfiArticleBlock, FfiCommentArea, FfiPicture, Res, RubiCore};

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    /// Opens a 专栏 / 图文 link (`opus/ID`, `read/cvN`, or a bare id) for reading.
    pub async fn article(&self, link: String) -> Res<FfiArticle> {
        let a = self.reading.open(&link).await?;
        let blocks = a
            .blocks
            .into_iter()
            .map(|b| match b {
                ArticleBlock::Heading(text) => FfiArticleBlock::Heading { text },
                ArticleBlock::Text(text) => FfiArticleBlock::Text { text },
                ArticleBlock::Quote(text) => FfiArticleBlock::Quote { text },
                ArticleBlock::Line => FfiArticleBlock::Line,
                ArticleBlock::Pictures(p) => FfiArticleBlock::Pictures {
                    pictures: p
                        .into_iter()
                        .map(|p| FfiPicture { url: p.url, width: p.width, height: p.height })
                        .collect(),
                },
            })
            .collect();
        Ok(FfiArticle {
            id: a.id,
            title: a.title,
            author_name: a.author_name,
            author_mid: a.author_mid,
            author_face: a.author_face,
            published_at: a.published_at,
            cover: a.cover,
            blocks,
            comments: (a.comment_kind > 0 && a.comment_oid > 0)
                .then_some(FfiCommentArea { kind: a.comment_kind, oid: a.comment_oid }),
            hidden: a.hidden as u32,
        })
    }
}
