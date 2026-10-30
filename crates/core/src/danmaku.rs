use rubi_api::{DanmakuApi, SEGMENT_MS};
use rubi_filter::danmaku::{DanmakuOptions, DanmakuOutput, process};

use crate::error::Result;

/// Loads every danmaku segment of `cid` plus its command danmaku and runs the filters.
pub async fn load_danmaku(
    api: &dyn DanmakuApi,
    cid: u64,
    duration_ms: u32,
    opts: &DanmakuOptions,
) -> Result<DanmakuOutput> {
    let view = api.view(cid).await?;
    let by_duration = duration_ms.div_ceil(SEGMENT_MS).max(1);
    let segments = if view.segment_count > 0 { view.segment_count } else { by_duration };
    let mut all = Vec::new();
    for i in 1..=segments {
        all.extend(api.segment(cid, i).await?);
    }
    Ok(process(all, view.commands, duration_ms, opts))
}
