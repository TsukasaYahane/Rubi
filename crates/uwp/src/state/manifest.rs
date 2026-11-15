//! A DASH manifest for the system player. B站 hands out one file per stream with its segment
//! index inside, which is DASH's on-demand profile; the manifest only says where things are.

use std::fmt::Write;

use rubi_core::Stream;

/// The manifest for `video` and `audio`; `None` when a stream lacks its segment ranges, which
/// the system player cannot do without. Either may stand alone: an audio-only track is still
/// a valid DASH presentation.
pub fn dash_manifest(duration_ms: u64, video: Option<&Stream>, audio: Option<&Stream>) -> Option<String> {
    let mut sets = String::new();
    if let Some(video) = video {
        sets.push_str(&adaptation_set("video/mp4", video)?);
    }
    if let Some(audio) = audio {
        sets.push_str(&adaptation_set("audio/mp4", audio)?);
    }
    if sets.is_empty() {
        return None;
    }
    let seconds = duration_ms as f64 / 1000.0;
    Some(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
         <MPD xmlns=\"urn:mpeg:dash:schema:mpd:2011\" profiles=\"urn:mpeg:dash:profile:isoff-on-demand:2011\" \
         type=\"static\" minBufferTime=\"PT1.5S\" mediaPresentationDuration=\"PT{seconds:.3}S\">\
         <Period>{sets}</Period></MPD>"
    ))
}

fn adaptation_set(mime: &str, s: &Stream) -> Option<String> {
    let ranges = s.segment.as_ref()?;
    let mut size = String::new();
    if s.width > 0 && s.height > 0 {
        let _ = write!(size, " width=\"{}\" height=\"{}\"", s.width, s.height);
    }
    Some(format!(
        "<AdaptationSet mimeType=\"{mime}\" segmentAlignment=\"true\" startWithSAP=\"1\">\
         <Representation id=\"{id}\" bandwidth=\"{bandwidth}\" codecs=\"{codecs}\"{size}>\
         <BaseURL>{url}</BaseURL>\
         <SegmentBase indexRange=\"{index}\"><Initialization range=\"{init}\"/></SegmentBase>\
         </Representation></AdaptationSet>",
        id = s.id,
        bandwidth = s.bandwidth,
        codecs = escape(&s.codecs),
        url = escape(&s.url),
        index = escape(&ranges.index),
        init = escape(&ranges.init),
    ))
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use rubi_core::SegmentRanges;

    use super::*;

    fn stream(id: u32, segment: bool) -> Stream {
        Stream {
            id,
            url: "https://a.b/v.m4s?x=1&y=2".into(),
            backup_urls: vec![],
            bandwidth: 1000,
            codecs: "avc1.640028".into(),
            width: 1920,
            height: 1080,
            segment: segment.then(|| SegmentRanges { init: "0-999".into(), index: "1000-2000".into() }),
        }
    }

    #[test]
    fn writes_both_streams_with_their_ranges() {
        let mpd = dash_manifest(12_500, Some(&stream(80, true)), Some(&stream(30280, true))).unwrap();
        assert!(mpd.contains("mediaPresentationDuration=\"PT12.500S\""));
        assert!(mpd.contains("mimeType=\"video/mp4\"") && mpd.contains("mimeType=\"audio/mp4\""));
        assert!(mpd.contains("indexRange=\"1000-2000\"") && mpd.contains("range=\"0-999\""));
        assert!(mpd.contains("v.m4s?x=1&amp;y=2"), "urls are escaped");
    }

    #[test]
    fn writes_audio_alone() {
        let mpd = dash_manifest(3_000, None, Some(&stream(30280, true))).unwrap();
        assert!(mpd.contains("mimeType=\"audio/mp4\""));
        assert!(!mpd.contains("video/mp4"), "no video set when only audio plays");
    }

    #[test]
    fn needs_the_segment_ranges() {
        assert!(dash_manifest(1, Some(&stream(80, false)), None).is_none());
        assert!(dash_manifest(1, Some(&stream(80, true)), Some(&stream(1, false))).is_none());
        assert!(dash_manifest(1, Some(&stream(80, true)), None).is_some());
        assert!(dash_manifest(1, None, None).is_none());
    }
}
