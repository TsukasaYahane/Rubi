//! Old zone ids (tid) to the v2 main regions the region feed takes.
//!
//! Profiles learn zones from cards, which carry old tids (a sub-zone such as 17 单机游戏, or a
//! parent). Since 2026-10 the region feed (`x/web-interface/region/feed/rcmd`) only takes v2 main
//! region ids. Parents and their subs come from the API collection's zone table; v2 ids from the
//! web channel config (`channelConfigMap` on www.bilibili.com/c/…), both read 2026-11-01.

/// Old parent tid → v2 main region.
const PARENTS: &[(u32, u32)] = &[
    (1, 1005),   // 动画 → 动画
    (13, 1005),  // 番剧
    (167, 1005), // 国创
    (3, 1003),   // 音乐
    (129, 1004), // 舞蹈
    (4, 1008),   // 游戏
    (36, 1010),  // 知识
    (188, 1012), // 科技 → 科技数码
    (234, 1018), // 运动 → 体育运动
    (223, 1013), // 汽车
    (160, 1030), // 生活 → 生活兴趣
    (211, 1020), // 美食
    (217, 1024), // 动物圈 → 动物
    (119, 1007), // 鬼畜
    (155, 1014), // 时尚 → 时尚美妆
    (202, 1009), // 资讯
    (5, 1002),   // 娱乐
    (181, 1001), // 影视
    (177, 1001), // 纪录片
    (23, 1001),  // 电影
    (11, 1001),  // 电视剧
];

/// Old sub-zone tids by parent.
const SUBS: &[(u32, &[u32])] = &[
    (1, &[24, 25, 47, 257, 210, 86, 253, 27]),
    (13, &[51, 152, 32, 33]),
    (167, &[153, 168, 169, 170, 195]),
    (3, &[28, 31, 30, 59, 193, 29, 130, 243, 244, 194]),
    (129, &[20, 154, 156, 198, 199, 200, 255]),
    (4, &[17, 171, 172, 65, 173, 121, 136, 19]),
    (36, &[201, 124, 228, 207, 208, 209, 229, 122, 39, 96, 98]),
    (188, &[95, 230, 231, 232, 233, 189, 190, 191]),
    (234, &[235, 249, 164, 236, 237, 238]),
    (223, &[258, 245, 246, 247, 248, 240, 227, 176, 224, 225, 226]),
    (160, &[138, 250, 251, 239, 161, 162, 21, 254, 163, 174]),
    (211, &[76, 212, 213, 214, 215]),
    (217, &[218, 219, 220, 221, 222, 75]),
    (119, &[22, 26, 126, 216, 127]),
    (155, &[157, 252, 158, 159, 192]),
    (202, &[203, 204, 205, 206]),
    (5, &[71, 241, 242, 137, 131]),
    (181, &[182, 183, 85, 184, 256]),
    (177, &[37, 178, 179, 180]),
    (23, &[147, 145, 146, 83]),
    (11, &[185, 187]),
];

/// The v2 main region an old tid (sub-zone or parent) belongs to. v2 main ids pass through.
pub fn main_region(tid: u32) -> Option<u32> {
    if (1001..1100).contains(&tid) {
        return Some(tid);
    }
    let parent = SUBS.iter().find(|(_, subs)| subs.contains(&tid)).map_or(tid, |(p, _)| *p);
    PARENTS.iter().find(|(p, _)| *p == parent).map(|(_, r)| *r)
}

/// Main regions for zones ranked by preference: mapped, deduplicated, best first.
pub fn main_regions(ranked_tids: impl IntoIterator<Item = u32>, limit: usize) -> Vec<u32> {
    let mut out = Vec::new();
    for r in ranked_tids.into_iter().filter_map(main_region) {
        if !out.contains(&r) {
            out.push(r);
            if out.len() == limit {
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_subs_and_parents() {
        assert_eq!(main_region(17), Some(1008)); // 单机游戏
        assert_eq!(main_region(65), Some(1008)); // 网络游戏
        assert_eq!(main_region(4), Some(1008));
        assert_eq!(main_region(138), Some(1030)); // 搞笑 → 生活兴趣
        assert_eq!(main_region(1010), Some(1010));
        assert_eq!(main_region(99_999), None);
        assert_eq!(main_regions([17, 65, 201, 138, 3], 3), vec![1008, 1010, 1030]);
    }
}
