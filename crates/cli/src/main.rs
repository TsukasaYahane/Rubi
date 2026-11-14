//! `rubi` — a terminal window into the Rust core, for tuning filters and the algorithm
//! before the Android UI exists. Data lives in `./.rubi-dev/` (git-ignored).

use std::fs;
use std::path::{Path, PathBuf};

use rubi_api::{QrState, Session};
use rubi_core::Services;
use rubi_model::{DislikeReason, Signal};

pub(crate) type Res<T> = Result<T, Box<dyn std::error::Error>>;

const USAGE: &str = "\
用法: rubi <命令> [参数]

  feed [n]                 刷新候选池并输出推荐流前 n 条（默认 20）
  why <bvid>               某条视频的打分明细
  log [n]                  最近 n 条被过滤内容（默认 30）
  signal <bvid> <信号>     模拟反馈: click like coin fav follow rewatch quick-exit
                           done:<0-1> dislike-up dislike-zone dislike-title
  dm <bvid>                拉取弹幕并显示过滤 / 空降合并结果
  play <bvid>              显示可用画质与音视频流
  login                    终端扫码登录（凭证明文存于 .rubi-dev，仅供开发）
  whoami                   当前登录状态
  export <file.zip> [--history]
  import <file.zip>

  comments <bvid> [time]   评论区（鱼骨图对话树，前 3 个对话展开）
  search <关键词>          搜索视频（已过滤广告）
  hot                      热搜（已过滤推广词）
  unread                   消息未读数                       [需登录，下同]
  inbox <reply|at|like>    消息中心
  chats                    私信会话（已过滤营销私信）
  chat <mid>               与某人的私信记录
  send <mid> <文本>        发送私信
  like|coin|later|follow <bvid>   互动操作（同时记入推荐画像）
  favs <bvid>              该视频所在 / 可加入的收藏夹
  history                  观看历史
  sync-follow              同步关注列表到推荐算法";

mod social;

fn data_dir() -> PathBuf {
    PathBuf::from(".rubi-dev")
}

fn load_session(dir: &Path) -> Session {
    fs::read_to_string(dir.join("session.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn save_session(dir: &Path, s: &Session) -> Res<()> {
    fs::write(dir.join("session.json"), serde_json::to_string_pretty(s)?)?;
    Ok(())
}

pub(crate) fn short(s: &str, n: usize) -> String {
    let mut out: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        out.push('…');
    }
    out
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Res<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first().map(String::as_str) else {
        println!("{USAGE}");
        return Ok(());
    };
    let dir = data_dir();
    fs::create_dir_all(&dir)?;
    let app = Services::open(&dir, load_session(&dir))?;
    app.account.ensure_device_ids().await?;
    save_session(&dir, &app.account.session())?;
    let arg = |i: usize| args.get(i).map(String::as_str);

    match cmd {
        "feed" => feed(&app, arg(1).and_then(|n| n.parse().ok()).unwrap_or(20)).await,
        "why" => why(&app, arg(1).ok_or("缺少 bvid")?),
        "log" => log(&app, arg(1).and_then(|n| n.parse().ok()).unwrap_or(30)),
        "signal" => signal(&app, arg(1).ok_or("缺少 bvid")?, arg(2).ok_or("缺少信号")?).await,
        "dm" => dm(&app, arg(1).ok_or("缺少 bvid")?).await,
        "dcup" => dcup(&app).await,
        "play" => play(&app, arg(1).ok_or("缺少 bvid")?).await,
        "login" => login(&app, &dir).await,
        "whoami" => {
            let nav = app.account.whoami().await?;
            println!("{}", if nav.is_login { format!("已登录: {} ({})", nav.uname, nav.mid) } else { "游客".into() });
            Ok(())
        }
        "export" => {
            let path = arg(1).ok_or("缺少文件路径")?;
            app.preferences.export_to(path, args.iter().any(|a| a == "--history"))?;
            println!("已导出到 {path}（不含登录凭证）");
            Ok(())
        }
        "import" => {
            app.preferences.import_from(arg(1).ok_or("缺少文件路径")?)?;
            println!("已导入");
            Ok(())
        }
        other => {
            if !social::run(&app, other, &args).await? {
                println!("{USAGE}");
            }
            Ok(())
        }
    }
}

async fn feed(app: &Services, n: usize) -> Res<()> {
    let r = app.home.refresh(20).await?;
    println!(
        "刷新: 拉取 {} · 保留 {} · 过滤 {} · 冷却跳过 {:?}",
        r.fetched.iter().map(|(k, n)| format!("{k:?}={n}")).collect::<Vec<_>>().join(" "),
        r.kept,
        r.blocked,
        r.skipped
    );
    for e in &r.errors {
        println!("  ! {e}");
    }
    println!();
    for (i, item) in app.home.next_page(n)?.iter().enumerate() {
        let c = &item.card;
        let top = item
            .score
            .parts
            .iter()
            .filter(|p| p.contribution.abs() > 0.01)
            .map(|p| format!("{:?}{:+.2}", p.factor, p.contribution))
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "{:>2}. {:+.3}  {}  [{}]  {}",
            i + 1,
            item.score.total,
            short(&c.title, 28),
            short(&c.owner.name, 8),
            c.bvid
        );
        println!("      {:?}  {}", c.sources, top);
    }
    Ok(())
}

fn why(app: &Services, bvid: &str) -> Res<()> {
    let Some(s) = app.home.explain(bvid)? else {
        println!("{bvid} 不在候选池里");
        return Ok(());
    };
    println!("总分 {:+.3}", s.total);
    for p in &s.parts {
        println!(
            "  {:<15} 原始 {:+.2} × 权重 {:.2} = {:+.3}",
            format!("{:?}", p.factor),
            p.raw,
            p.weight,
            p.contribution
        );
    }
    println!("  来源调整 {:+.3} · 过滤惩罚 {:+.3}", s.source_adjust, s.penalty_adjust);
    Ok(())
}

fn log(app: &Services, n: usize) -> Res<()> {
    for e in app.home.filter_log(n)? {
        println!("{:<14} {:<26} {:<16} {}", e.category, e.rule_id, e.source, short(&e.title, 30));
    }
    Ok(())
}

async fn signal(app: &Services, bvid: &str, kind: &str) -> Res<()> {
    let card = app.videos.card(bvid).await?;
    let signal = match kind {
        "click" => Signal::Click,
        "like" => Signal::Like,
        "coin" => Signal::Coin,
        "fav" => Signal::Favorite,
        "follow" => Signal::Follow,
        "rewatch" => Signal::Rewatch,
        "quick-exit" => Signal::QuickExit,
        "dislike-up" => Signal::Dislike { reason: DislikeReason::Up },
        "dislike-zone" => Signal::Dislike { reason: DislikeReason::Zone },
        "dislike-title" => Signal::Dislike { reason: DislikeReason::Title },
        k if k.starts_with("done:") => Signal::Completion { ratio: k[5..].parse()? },
        _ => return Err(format!("未知信号 {kind}").into()),
    };
    app.home.record(bvid, signal).await?;
    println!("已记录 {kind} → {}（标签 {:?}）", short(&card.title, 30), card.tags);
    Ok(())
}

async fn dcup(app: &Services) -> Res<()> {
    let ups = app.following.ups().await?;
    let Some(first) = ups.first() else { println!("没有关注"); return Ok(()) };
    let (posts, _) = app.following.up_posts(first.mid, None).await?;
    println!("up {} 动态 {} 条", first.name, posts.len());
    for post in posts.iter().take(6) {
        println!(
            "  id={} kind={:?} ckind={} coid={} count={}",
            post.id, post.kind, post.comment_kind, post.comment_oid, post.comment_count
        );
        if post.comment_oid > 0 {
            let area = rubi_core::CommentArea { kind: post.comment_kind, oid: post.comment_oid };
            match app.comments.page(area, false, None).await {
                Ok(page) => println!("    → {} 楼", page.threads.len()),
                Err(e) => println!("    → 错 {e}"),
            }
        }
    }
    Ok(())
}

async fn dm(app: &Services, bvid: &str) -> Res<()> {
    let card = app.videos.detail(bvid).await?.card;
    let out = app.danmaku.all(card.cid.unwrap_or_default(), card.duration_s * 1000).await?;
    println!("{} · 时长 {}s", card.title, card.duration_s);
    println!("保留 {} 条", out.kept.len());
    for (cat, n) in &out.removed {
        println!("  移除 {cat:?}: {n}");
    }
    for m in &out.jump_marks {
        println!("  空降点 {}", m.label);
    }
    for d in out.kept.iter().take(10) {
        println!("  {:>6.1}s  {}", d.progress_ms as f32 / 1000.0, short(&d.content, 30));
    }
    Ok(())
}

async fn play(app: &Services, bvid: &str) -> Res<()> {
    let card = app.videos.card(bvid).await?;
    let playback = app.videos.playback(bvid, 80).await?;
    let p = &playback.url;
    println!("{} · 当前画质 {} · 可选 {:?}", card.title, p.quality, p.accept);
    for v in &p.video {
        let host = |u: &str| u.split('/').nth(2).unwrap_or_default().to_owned();
        let backups: Vec<String> = v.backup_urls.iter().map(|u| host(u)).collect();
        println!(
            "  视频 {} {}x{} {} {}kbps  {} ← {:?}",
            v.id,
            v.width,
            v.height,
            v.codecs,
            v.bandwidth / 1000,
            host(&v.url),
            backups
        );
    }
    for a in &p.audio {
        println!("  音频 {} {} {}kbps", a.id, a.codecs, a.bandwidth / 1000);
    }
    println!("  请求头 {:?}", playback.headers);
    Ok(())
}

async fn login(app: &Services, dir: &Path) -> Res<()> {
    let ticket = app.account.qr_generate().await?;
    let code = qrcode::QrCode::new(ticket.url.as_bytes())?;
    println!("{}", code.render::<qrcode::render::unicode::Dense1x2>().quiet_zone(true).build());
    println!("用手机端 App 扫码登录（180 秒内有效）");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        match app.account.qr_poll(&ticket.key).await? {
            QrState::Waiting => {}
            QrState::Scanned => println!("已扫码，请在手机上确认"),
            QrState::Expired => return Err("二维码已过期".into()),
            QrState::Success(session) => {
                save_session(dir, &session)?;
                println!("登录成功（凭证明文存于 {}，仅供开发调试）", dir.join("session.json").display());
                return Ok(());
            }
        }
    }
}
