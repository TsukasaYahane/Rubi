//! Text with B站 emotes drawn inline: a `RichTextBlock` whose paragraph holds the text's runs and,
//! for each `[token]` with a known image, that image at about the text's height. Comments, posts
//! and articles show text this way.

use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use windows_collections::IVector;
use windows_core::{Interface, Result};

use crate::state::emotes::{Piece, pieces};
use crate::state::settings::escape;
use crate::ui::load_object;
use crate::ui::shell::shell;
use crate::xaml::{Block, IRichTextBlock};

/// The common emote set, once read.
static COMMON: OnceLock<HashMap<String, String>> = OnceLock::new();
static ASKED: AtomicBool = AtomicBool::new(false);

/// Starts reading the common emote set, once a run: text shown before it comes keeps their tokens.
pub fn read_common() {
    if COMMON.get().is_some() || ASKED.swap(true, Ordering::Relaxed) {
        return;
    }
    let Some(backend) = shell().and_then(|s| s.backend.get().cloned()) else {
        ASKED.store(false, Ordering::Relaxed);
        return;
    };
    let core = backend.clone();
    backend.spawn(async move {
        match core.services.emotes.all().await {
            Ok(list) => {
                let _ = COMMON.set(list.iter().map(|e| (e.text.clone(), e.url.clone())).collect());
            }
            // Asked again with the next text.
            Err(_) => ASKED.store(false, Ordering::Relaxed),
        }
    });
}

/// The image of `token`: the text's own first, then the common set.
pub fn lookup<'a>(own: &'a BTreeMap<String, String>) -> impl Fn(&str) -> Option<&'a str> {
    read_common();
    move |token| own.get(token).or_else(|| COMMON.get()?.get(token)).map(String::as_str)
}

/// The inlines of `text` in XAML, emotes `size` px square, for a `Paragraph`.
pub fn inlines<'a>(text: &'a str, lookup: impl Fn(&str) -> Option<&'a str>, size: f64) -> String {
    let mut out = String::new();
    for piece in pieces(text, lookup) {
        match piece {
            Piece::Text(run) => {
                for (is_link, part) in split_links(run) {
                    if is_link {
                        out.push_str(&format!(
                            r##"<Hyperlink NavigateUri="{}"><Run Text="{}" Foreground="#0050EF" /></Hyperlink>"##,
                            escape(part),
                            escape(&link_label(part))
                        ));
                        continue;
                    }
                    for (at, line) in part.split('\n').enumerate() {
                        if at > 0 {
                            out.push_str("<LineBreak />");
                        }
                        if !line.is_empty() {
                            out.push_str(&format!(r#"<Run Text="{}" />"#, escape(line)));
                        }
                    }
                }
            }
            Piece::Emote { token, url } => out.push_str(&format!(
                r#"<InlineUIContainer><Image Source="{}" Width="{size}" Height="{size}" Margin="1,0,1,{}" AutomationProperties.Name="{}" /></InlineUIContainer>"#,
                escape(url),
                -(size * 0.22).round(),
                escape(token)
            )),
        }
    }
    out
}

/// Shows `text` in `rich`, with `own`'s emotes and the common ones, `size` px square.
pub fn show(rich: &IRichTextBlock, text: &str, own: &BTreeMap<String, String>, size: f64) -> Result<()> {
    let paragraph = format!(
        r#"<Paragraph xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">{}</Paragraph>"#,
        inlines(text, lookup(own), size)
    );
    let blocks = rich.Blocks()?.cast::<IVector<Block>>()?;
    blocks.Clear()?;
    blocks.Append(&load_object(&paragraph)?.cast::<Block>()?)
}

/// Cuts `run` into plain and link pieces: a link ends at the next blank or 中文 punctuation.
fn split_links(run: &str) -> Vec<(bool, &str)> {
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(found) = run[at..].find("http") {
        let start = at + found;
        let rest = &run[start..];
        if !(rest.starts_with("https://") || rest.starts_with("http://")) {
            at = start + 4;
            continue;
        }
        let end = rest.find(|c: char| c.is_whitespace() || "，。！？；、）】".contains(c)).map_or(run.len(), |i| start + i);
        if start > at {
            out.push((false, &run[at..start]));
        }
        out.push((true, &run[start..end]));
        at = end;
    }
    if at < run.len() {
        out.push((false, &run[at..]));
    }
    out
}

/// What a link reads as: its host, and an ellipsis when there is a path behind it.
fn link_label(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let host = rest.split('/').next().unwrap_or(rest);
    if rest.contains('/') { format!("{host}/…") } else { host.to_owned() }
}
