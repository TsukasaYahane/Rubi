//! Export / import of settings, rules, weights and the interest profile as one zip,
//! so a profile can move to another device. Login credentials are never included.

use std::io::{Read, Seek, Write};

use rubi_filter::RuleSet;
use rubi_model::FeedbackEvent;
use rubi_recommend::InterestProfile;
use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;

use crate::error::{CoreError, Result};
use crate::feed::Settings;
use rubi_model::VideoCard;

const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Default)]
pub struct ExportOptions {
    pub include_history: bool,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    format: u32,
    app: String,
    exported_at: i64,
    has_history: bool,
}

/// The settings without the rule table, which is written as TOML on its own.
#[derive(Serialize, Deserialize)]
struct SettingsFile {
    weights: rubi_recommend::Weights,
    source_weights: rubi_recommend::SourceWeights,
    candidate_ttl_s: i64,
    min_source_interval_s: u64,
    #[serde(default)]
    danmaku_display: crate::feed::DanmakuDisplay,
    #[serde(default)]
    background_play: bool,
    #[serde(default)]
    preview: crate::feed::PreviewSettings,
    #[serde(default = "default_speed")]
    long_press_speed: f32,
    #[serde(default = "default_quality")]
    preferred_quality: u32,
    #[serde(default)]
    auto_pip: bool,
}

fn default_speed() -> f32 {
    Settings::default().long_press_speed
}

fn default_quality() -> u32 {
    Settings::default().preferred_quality
}

pub fn export_bundle<W: Write + Seek>(
    out: W,
    settings: &Settings,
    profile: &InterestProfile,
    history: &[FeedbackEvent],
    playlist: &[VideoCard],
    opts: ExportOptions,
) -> Result<()> {
    let mut zip = zip::ZipWriter::new(out);
    let o = SimpleFileOptions::default();
    let manifest = Manifest {
        format: FORMAT_VERSION,
        app: "rubi".into(),
        exported_at: rubi_model::now_secs(),
        has_history: opts.include_history,
    };
    zip.start_file("manifest.json", o)?;
    zip.write_all(&serde_json::to_vec_pretty(&manifest)?)?;
    zip.start_file("rules.toml", o)?;
    zip.write_all(settings.rules.to_toml()?.as_bytes())?;
    let sf = SettingsFile {
        weights: settings.weights.clone(),
        source_weights: settings.source_weights.clone(),
        candidate_ttl_s: settings.candidate_ttl_s,
        min_source_interval_s: settings.min_source_interval_s,
        danmaku_display: settings.danmaku_display.clone(),
        background_play: settings.background_play,
        preview: settings.preview.clone(),
        long_press_speed: settings.long_press_speed,
        preferred_quality: settings.preferred_quality,
        auto_pip: settings.auto_pip,
    };
    zip.start_file("settings.json", o)?;
    zip.write_all(&serde_json::to_vec_pretty(&sf)?)?;
    if !playlist.is_empty() {
        // Without the raw API JSON: the list needs the card, not what it was parsed from.
        let slim: Vec<VideoCard> =
            playlist.iter().map(|c| VideoCard { raw: serde_json::Value::Null, ..c.clone() }).collect();
        zip.start_file("playlist.json", o)?;
        zip.write_all(&serde_json::to_vec_pretty(&slim)?)?;
    }
    zip.start_file("profile.json", o)?;
    zip.write_all(&serde_json::to_vec(profile)?)?;
    if opts.include_history {
        zip.start_file("history.jsonl", o)?;
        for ev in history {
            zip.write_all(&serde_json::to_vec(ev)?)?;
            zip.write_all(b"\n")?;
        }
    }
    zip.finish()?;
    Ok(())
}

pub struct Imported {
    pub settings: Settings,
    pub profile: InterestProfile,
    pub history: Vec<FeedbackEvent>,
    /// Empty for exports made before the play list existed.
    pub playlist: Vec<VideoCard>,
}

pub fn import_bundle<R: Read + Seek>(input: R) -> Result<Imported> {
    let mut zip = zip::ZipArchive::new(input)?;
    let mut read = |name: &str| -> Result<Option<String>> {
        match zip.by_name(name) {
            Ok(mut f) => {
                let mut s = String::new();
                f.read_to_string(&mut s)?;
                Ok(Some(s))
            }
            Err(zip::result::ZipError::FileNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    };
    let manifest: Manifest =
        serde_json::from_str(&read("manifest.json")?.ok_or_else(|| CoreError::Invalid("not a Rubi export".into()))?)?;
    if manifest.format > FORMAT_VERSION {
        return Err(CoreError::Invalid(format!("export format {} is newer than this app", manifest.format)));
    }
    let mut rules = read("rules.toml")?.map(|t| RuleSet::from_toml(&t)).transpose()?.unwrap_or_else(RuleSet::builtin);
    rules.merge_builtin();
    let mut settings = Settings { rules, ..Settings::default() };
    if let Some(s) = read("settings.json")? {
        let sf: SettingsFile = serde_json::from_str(&s)?;
        settings.weights = sf.weights;
        settings.source_weights = sf.source_weights;
        settings.candidate_ttl_s = sf.candidate_ttl_s;
        settings.min_source_interval_s = sf.min_source_interval_s;
        settings.danmaku_display = sf.danmaku_display;
        settings.background_play = sf.background_play;
        settings.preview = sf.preview;
        settings.long_press_speed = sf.long_press_speed;
        settings.preferred_quality = sf.preferred_quality;
        settings.auto_pip = sf.auto_pip;
    }
    let profile = read("profile.json")?.map(|p| serde_json::from_str(&p)).transpose()?.unwrap_or_default();
    let history = read("history.jsonl")?
        .map(|h| {
            h.lines()
                .filter(|l| !l.trim().is_empty())
                .map(serde_json::from_str)
                .collect::<std::result::Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let playlist: Vec<VideoCard> =
        read("playlist.json")?.map(|p| serde_json::from_str(&p)).transpose()?.unwrap_or_default();
    Ok(Imported { settings, profile, history, playlist })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubi_model::Signal;
    use std::io::Cursor;

    #[test]
    fn round_trips_without_credentials() {
        let mut settings = Settings::default();
        settings.rules.up_blacklist.push(42);
        let profile = InterestProfile { half_life_days: 14.0, ..Default::default() };
        let history = vec![FeedbackEvent { signal: Signal::Like, item: None, at: 1 }];

        let mut buf = Cursor::new(Vec::new());
        let playlist = vec![VideoCard { bvid: "BV1".into(), raw: serde_json::json!({"big": 1}), ..Default::default() }];
        export_bundle(&mut buf, &settings, &profile, &history, &playlist, ExportOptions { include_history: true })
            .unwrap();
        let bytes = buf.into_inner();
        let text = String::from_utf8_lossy(&bytes);
        assert!(!text.contains("SESSDATA"));

        let back = import_bundle(Cursor::new(bytes)).unwrap();
        assert_eq!(back.settings.rules.up_blacklist, vec![42]);
        assert_eq!(back.profile.half_life_days, 14.0);
        assert_eq!(back.history, history);
        assert_eq!(back.playlist.len(), 1);
        assert_eq!(back.playlist[0].bvid, "BV1");
        assert!(back.playlist[0].raw.is_null(), "raw API JSON is not exported");
    }
}
