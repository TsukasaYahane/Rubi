//! Commands for comments, search, the message center, chat and account actions.

use rubi_api::UpOrder;
use rubi_core::{Services, ThreadView};
use rubi_model::NoticeKind;

use crate::{Res, short};

/// Returns false when `cmd` is not one of these commands.
pub async fn run(app: &Services, cmd: &str, args: &[String]) -> Res<bool> {
    let arg = |i: usize| args.get(i).map(String::as_str);
    let bvid = || arg(1).ok_or("缺少 bvid");
    match cmd {
        "comments" => comments(app, bvid()?, arg(2) == Some("time")).await?,
        "search" => search(app, &args[1..].join(" ")).await?,
        "hot" => {
            let (words, hidden) = app.discover.hot_words(10).await?;
            for (i, w) in words.iter().enumerate() {
                println!("{:>2}  {}", i + 1, w.keyword);
            }
            println!("（已过滤推广热搜 {hidden} 条）");
        }
        "unread" => {
            let u = app.messages.unread().await?;
            println!("回复 {} · @我 {} · 赞 {} · 私信 {}", u.reply, u.at, u.like, u.chat);
        }
        "inbox" => {
            let kind = match arg(1).unwrap_or("reply") {
                "at" => NoticeKind::At,
                "like" => NoticeKind::Like,
                _ => NoticeKind::Reply,
            };
            let (items, _) = app.messages.notices(kind, None).await?;
            for n in items {
                println!("{}  {}\n    {}", n.from, short(&n.text, 40), short(&n.context, 50));
            }
        }
        "chats" => {
            let (sessions, hidden) = app.messages.sessions().await?;
            for s in sessions {
                let who = if s.last_from_me { "我：" } else { "" };
                println!(
                    "{:<12} 未读 {:<3} {}{}  [mid {}]",
                    short(&s.name, 10),
                    s.unread,
                    who,
                    short(&s.last_text, 30),
                    s.talker_mid
                );
            }
            println!("（已过滤营销私信 {hidden} 个会话）");
        }
        "chat" => {
            let mid: u64 = arg(1).ok_or("缺少 mid")?.parse()?;
            for m in app.messages.chat(mid).await? {
                let who = if m.sender_mid == mid { "对方" } else { "我" };
                println!("{who}: {}", m.text);
            }
        }
        "send" => {
            let mid: u64 = arg(1).ok_or("缺少 mid")?.parse()?;
            let text = args.get(2..).map(|a| a.join(" ")).filter(|t| !t.is_empty()).ok_or("缺少文本")?;
            app.messages.send(mid, &text).await?;
            println!("已发送");
        }
        "like" | "coin" | "later" | "follow" => {
            let bvid = bvid()?;
            match cmd {
                "like" => app.actions.like(bvid, true).await?,
                "coin" => app.actions.coin(bvid, 1, false).await?,
                "later" => app.actions.watch_later(bvid).await?,
                _ => app.actions.follow_up(bvid, true).await?,
            }
            println!("完成：{cmd} {}", short(&app.videos.card(bvid).await?.title, 30));
        }
        "up" => {
            let mid: u64 = arg(1).ok_or("缺少 mid")?.parse()?;
            let order = match arg(2) {
                Some("click") => UpOrder::Click,
                Some("stow") => UpOrder::Stow,
                _ => UpOrder::Pubdate,
            };
            let p = app.space.profile(mid).await?;
            let followed = if p.followed { " · 已关注" } else { "" };
            println!(
                "{} Lv{} {}  粉丝 {} · 关注 {} · 投稿 {}{followed}",
                p.name, p.level, p.official, p.fans, p.followings, p.archives
            );
            println!("{}", p.sign);
            let page = app.space.uploads(mid, 1, order).await?;
            for c in &page.items {
                println!("{:>8} 播放  {}  [{}]", c.stats.view, short(&c.title, 30), c.bvid);
            }
            println!(
                "共 {} 个 · 本页隐藏充电专属 {} 个 · 还有更多：{}",
                page.total, page.hidden_charging, page.has_more
            );
        }
        "favs" => {
            for f in app.actions.fav_folders(bvid()?).await? {
                println!("{} {:<20} {} 个  [id {}]", if f.contains { "■" } else { "□" }, f.title, f.count, f.id);
            }
        }
        "history" => {
            let (items, _) = app.library.history(None).await?;
            for h in items {
                let p = if h.progress_s < 0 { "看完".to_string() } else { format!("{}s", h.progress_s) };
                println!("{:<8} {}  [{}]", p, short(&h.card.title, 30), h.card.bvid);
            }
        }
        "sync-follow" => println!("已同步 {} 个关注", app.following.sync_to_profile().await?),
        _ => return Ok(false),
    }
    Ok(true)
}

async fn search(app: &Services, keyword: &str) -> Res<()> {
    if keyword.trim().is_empty() {
        return Err("缺少关键词".into());
    }
    let (cards, hidden) = app.discover.search(keyword, 1).await?;
    for c in &cards {
        println!("{}  [{}]  {} 播放  {}", short(&c.title, 30), short(&c.owner.name, 8), c.stats.view, c.bvid);
    }
    println!("（{} 条，已过滤 {hidden} 条）", cards.len());
    Ok(())
}

async fn comments(app: &Services, bvid: &str, by_time: bool) -> Res<()> {
    let card = app.videos.card(bvid).await?;
    let page = app.comments.page(card.aid, by_time, None).await?;
    println!("{} · 已隐藏置顶带货评论 {} 条\n", card.title, page.hidden_pinned);
    for p in &page.pinned {
        println!("[置顶] {}: {}", p.name, short(&p.text, 50));
    }
    for (i, t) in page.threads.iter().enumerate() {
        // The first three threads are loaded in full; the rest show their previews.
        let shown =
            if i < 3 && t.total_replies > 0 { app.comments.thread(card.aid, t.root_rpid, 6).await? } else { t.clone() };
        print_thread(&shown);
        println!();
    }
    Ok(())
}

/// Draws a thread as a fishbone tree with box-drawing characters.
fn print_thread(thread: &ThreadView) {
    let mut open: Vec<bool> = Vec::new();
    for (row, c) in &thread.rows {
        let up = if c.is_up { " [UP]" } else { "" };
        let to = row.reply_to.as_deref().map(|n| format!("回复 @{n}：")).unwrap_or_default();
        if row.depth == 0 {
            println!("{}{up}  赞 {}\n  {}", c.name, c.likes, short(&c.text, 60));
            continue;
        }
        let d = row.depth as usize;
        open.resize(d + 1, false);
        let lead: String = (1..d).map(|l| if open[l] { "│  " } else { "   " }).collect();
        let branch = if row.has_later_sibling { "├─ " } else { "└─ " };
        println!("{lead}{branch}{}{up}: {to}{}", c.name, short(&c.text, 50));
        open[d] = row.has_later_sibling;
    }
    if thread.hidden > 0 {
        println!("   └─ … 还有 {} 条（再看 5 条）", thread.hidden);
    }
}
