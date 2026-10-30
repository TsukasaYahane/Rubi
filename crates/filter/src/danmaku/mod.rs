//! Danmaku processing: drops command danmaku, applies keyword / user blocks,
//! merges 空降 danmaku and lays the rest out on tracks.

pub mod jump;
pub mod layout;

use std::collections::{HashMap, HashSet};

use regex::Regex;
use rubi_model::{CommandDanmaku, Danmaku, DanmakuMode, JumpMark};

use crate::pipeline::Category;

#[derive(Debug, Clone, Default)]
pub struct DanmakuOptions {
    pub drop_commands: bool,
    pub show_advanced: bool,
    pub merge_jump: bool,
    pub keywords: Vec<Regex>,
    pub blocked_users: HashSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DanmakuOutput {
    /// Danmaku to render, sorted by progress.
    pub kept: Vec<Danmaku>,
    /// Command danmaku still to render (empty when they are dropped).
    pub commands: Vec<CommandDanmaku>,
    pub jump_marks: Vec<JumpMark>,
    /// How many items each category removed.
    pub removed: HashMap<Category, usize>,
}

/// Runs every danmaku filter in order. `duration_ms` is the video length.
pub fn process(
    dms: Vec<Danmaku>,
    commands: Vec<CommandDanmaku>,
    duration_ms: u32,
    opts: &DanmakuOptions,
) -> DanmakuOutput {
    let mut removed: HashMap<Category, usize> = HashMap::new();
    let mut count = |c: Category| *removed.entry(c).or_default() += 1;

    let commands = if opts.drop_commands {
        for _ in &commands {
            count(Category::CommandDanmaku);
        }
        Vec::new()
    } else {
        commands
    };

    let mut kept: Vec<Danmaku> = Vec::with_capacity(dms.len());
    for d in dms {
        let advanced = matches!(d.kind(), DanmakuMode::Advanced | DanmakuMode::Code | DanmakuMode::Bas);
        if advanced && !opts.show_advanced {
            count(Category::AdvancedDanmaku);
        } else if opts.blocked_users.contains(&d.mid_hash) {
            count(Category::DanmakuUser);
        } else if opts.keywords.iter().any(|k| k.is_match(&d.content)) {
            count(Category::DanmakuKeyword);
        } else {
            kept.push(d);
        }
    }

    let mut jump_marks = Vec::new();
    if opts.merge_jump {
        let merged = jump::merge(&kept, duration_ms);
        let dropped: HashSet<u64> = merged.dropped.iter().copied().collect();
        for _ in &merged.dropped {
            count(Category::JumpMerge);
        }
        kept.retain(|d| !dropped.contains(&d.id));
        jump_marks = merged.marks;
    }

    kept.sort_by_key(|d| d.progress_ms);
    DanmakuOutput { kept, commands, jump_marks, removed }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dm(id: u64, progress_ms: u32, mode: u8, content: &str, ctime: i64) -> Danmaku {
        Danmaku {
            id,
            progress_ms,
            mode,
            content: content.into(),
            ctime,
            mid_hash: format!("u{id}"),
            ..Default::default()
        }
    }

    #[test]
    fn full_pass() {
        let dms = vec![
            dm(1, 1_000, 1, "前排", 1),
            dm(2, 2_000, 1, "空降 05:32", 5),
            dm(3, 3_000, 1, "空降5:30", 2),
            dm(4, 4_000, 7, "[0,0,1,\"高级\"]", 3),
            dm(5, 5_000, 1, "剧透警告", 4),
            dm(6, 6_000, 1, "被屏蔽的人", 6),
        ];
        let cmds = vec![CommandDanmaku { id: 9, command: "#ATTENTION#".into(), ..Default::default() }];
        let opts = DanmakuOptions {
            drop_commands: true,
            show_advanced: false,
            merge_jump: true,
            keywords: vec![Regex::new("剧透").unwrap()],
            blocked_users: ["u6".to_string()].into_iter().collect(),
        };
        let out = process(dms, cmds, 1_694_000, &opts);
        let ids: Vec<u64> = out.kept.iter().map(|d| d.id).collect();
        assert_eq!(ids, vec![1, 3], "earliest-sent jump danmaku (ctime 2) is kept");
        assert!(out.commands.is_empty());
        assert_eq!(out.jump_marks.len(), 1);
        assert_eq!(out.removed[&Category::JumpMerge], 1);
        assert_eq!(out.removed[&Category::AdvancedDanmaku], 1);
        assert_eq!(out.removed[&Category::DanmakuKeyword], 1);
        assert_eq!(out.removed[&Category::DanmakuUser], 1);
        assert_eq!(out.removed[&Category::CommandDanmaku], 1);
    }
}
