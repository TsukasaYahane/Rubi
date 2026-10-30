//! Tolerant JSON → model conversion. The various list endpoints share most fields
//! but differ in names and types, so every accessor accepts the known variants.

use rubi_model::{Owner, SourceKind, Stats, VideoCard, Zone};
use serde_json::Value;

pub fn u64_of(v: &Value) -> u64 {
    match v {
        Value::Number(n) => n.as_u64().or_else(|| n.as_f64().map(|f| f.max(0.0) as u64)).unwrap_or(0),
        Value::String(s) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

pub fn str_of(v: &Value) -> String {
    v.as_str().unwrap_or_default().to_owned()
}

/// Accepts seconds as a number or a `mm:ss` / `hh:mm:ss` string.
pub fn duration_of(v: &Value) -> u32 {
    match v {
        Value::String(s) => {
            s.split(':').try_fold(0u32, |acc, p| p.trim().parse::<u32>().ok().map(|n| acc * 60 + n)).unwrap_or(0)
        }
        other => u64_of(other) as u32,
    }
}

fn stats_of(s: &Value) -> Stats {
    let pick = |names: &[&str]| names.iter().map(|n| u64_of(&s[*n])).find(|&n| n > 0).unwrap_or(0);
    Stats {
        view: pick(&["view", "play", "vv"]),
        like: pick(&["like"]),
        coin: pick(&["coin"]),
        favorite: pick(&["favorite", "favorites"]),
        danmaku: pick(&["danmaku", "dm"]),
        reply: pick(&["reply"]),
    }
}

/// Converts an archive-shaped object (home rcmd item, popular, ranking, related, region…).
pub fn card_of(v: &Value, source: SourceKind) -> VideoCard {
    let owner = &v["owner"];
    let zone = match (u64_of(&v["tid"]), str_of(&v["tname"])) {
        (0, _) => None,
        (tid, name) => Some(Zone { tid: tid as u32, name }),
    };
    let aid = [&v["aid"], &v["id"]].into_iter().map(u64_of).find(|&n| n > 0).unwrap_or(0);
    VideoCard {
        bvid: str_of(&v["bvid"]),
        aid,
        cid: Some(u64_of(&v["cid"])).filter(|&c| c > 0),
        title: str_of(&v["title"]),
        desc: str_of(&v["desc"]),
        cover: [&v["pic"], &v["cover"]].into_iter().map(str_of).find(|s| !s.is_empty()).unwrap_or_default(),
        owner: Owner { mid: u64_of(&owner["mid"]), name: str_of(&owner["name"]) },
        zone,
        tags: Vec::new(),
        duration_s: duration_of(&v["duration"]),
        pubdate: [&v["pubdate"], &v["ctime"]].into_iter().map(|x| u64_of(x) as i64).find(|&n| n > 0).unwrap_or(0),
        stats: stats_of(&v["stat"]),
        sources: vec![source],
        raw: v.clone(),
    }
}

/// Converts a video item of the dynamic feed (`/x/polymer/web-dynamic/v1/feed/all`).
pub fn card_of_dynamic(item: &Value) -> Option<VideoCard> {
    let modules = &item["modules"];
    let archive = &modules["module_dynamic"]["major"]["archive"];
    if archive.is_null() {
        return None;
    }
    let author = &modules["module_author"];
    Some(VideoCard {
        bvid: str_of(&archive["bvid"]),
        aid: u64_of(&archive["aid"]),
        title: str_of(&archive["title"]),
        desc: str_of(&archive["desc"]),
        cover: str_of(&archive["cover"]),
        owner: Owner { mid: u64_of(&author["mid"]), name: str_of(&author["name"]) },
        duration_s: duration_of(&archive["duration_text"]),
        pubdate: u64_of(&author["pub_ts"]) as i64,
        stats: Stats {
            view: u64_of(&archive["stat"]["play"]),
            danmaku: u64_of(&archive["stat"]["danmaku"]),
            ..Default::default()
        },
        sources: vec![SourceKind::FollowedUps],
        raw: item.clone(),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_rcmd_item() {
        let v = json!({
            "id": 170001, "bvid": "BV17x411w7KC", "cid": 279786, "goto": "av", "title": "t",
            "pic": "http://i0.hdslb.com/a.jpg", "duration": 125, "pubdate": 1700000000,
            "owner": {"mid": 2, "name": "up"}, "stat": {"view": 1000, "like": 30, "danmaku": 4}
        });
        let c = card_of(&v, SourceKind::BiliRecommend);
        assert_eq!((c.aid, c.cid, c.duration_s, c.stats.view, c.owner.mid), (170001, Some(279786), 125, 1000, 2));
        assert_eq!(c.raw["goto"], "av");
    }

    #[test]
    fn parses_string_durations() {
        assert_eq!(duration_of(&json!("01:02:03")), 3723);
        assert_eq!(duration_of(&json!("4:05")), 245);
        assert_eq!(duration_of(&json!(null)), 0);
    }
}
