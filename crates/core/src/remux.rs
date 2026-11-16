//! 转封装: the fragmented MP4 that B站 sends as `.m4s` rewritten as an ordinary M4A. Nothing is
//! decoded — the samples move as they are — but the sample tables a fragment-only file keeps in
//! each `moof` have to be gathered into the single `moov` the rest of the world reads.

use std::fs::File;
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// A box inside a run of boxes: what it is, where its body starts, and where it ends.
struct Child {
    kind: [u8; 4],
    start: usize,
    body: usize,
    end: usize,
}

fn children(bytes: &[u8]) -> Vec<Child> {
    let mut out = Vec::new();
    let mut at = 0;
    while at + 8 <= bytes.len() {
        let mut size = u32_at(bytes, at) as usize;
        let kind = [bytes[at + 4], bytes[at + 5], bytes[at + 6], bytes[at + 7]];
        let mut header = 8;
        if size == 1 {
            size = u64_at(bytes, at + 8) as usize;
            header = 16;
        } else if size == 0 {
            size = bytes.len() - at;
        }
        if size < header || at + size > bytes.len() {
            break;
        }
        out.push(Child { kind, start: at, body: at + header, end: at + size });
        at += size;
    }
    out
}

fn child(children: &[Child], kind: [u8; 4]) -> Option<&Child> {
    children.iter().find(|c| c.kind == kind)
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    bytes.get(at..at + 4).and_then(|b| b.try_into().ok()).map(u32::from_be_bytes).unwrap_or(0)
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    bytes.get(at..at + 8).and_then(|b| b.try_into().ok()).map(u64::from_be_bytes).unwrap_or(0)
}

/// A box built here.
fn boxed(kind: [u8; 4], body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len() + 8);
    out.extend_from_slice(&((body.len() as u32 + 8).to_be_bytes()));
    out.extend_from_slice(&kind);
    out.extend_from_slice(body);
    out
}

fn full_box(kind: [u8; 4], version: u8, flags: u32, body: &[u8]) -> Vec<u8> {
    let mut inner = Vec::with_capacity(body.len() + 4);
    inner.push(version);
    inner.extend_from_slice(&flags.to_be_bytes()[1..]);
    inner.extend_from_slice(body);
    boxed(kind, &inner)
}

/// One sample of the track: where its bytes are in the source, and how long it lasts.
struct Sample {
    offset: u64,
    size: u32,
    duration: u32,
}

/// Reads one `moof`'s sample table, pointing at the bytes inside its `mdat`.
fn read_moof(bytes: &[u8], base: u64, samples: &mut Vec<Sample>) {
    let outer = children(bytes);
    let Some(moof) = outer.first() else { return };
    if &moof.kind != b"moof" {
        return;
    }
    let inside = moof.body;
    let inner = children(&bytes[inside..moof.end]);
    let Some(traf) = child(&inner, *b"traf") else { return };
    let traf_base = inside + traf.body;
    let fields = children(&bytes[traf_base..inside + traf.end]);
    let Some(tfhd) = child(&fields, *b"tfhd") else { return };
    let tfhd_body = traf_base + tfhd.body;
    let flags = u32_at(bytes, tfhd_body) & 0x00ff_ffff;
    // Past the flags and the track id, where the defaults start.
    let mut cursor = tfhd_body + 8;
    let mut default_duration = 0;
    let mut default_size = 0;
    if flags & 0x000001 != 0 {
        cursor += 8;
    }
    if flags & 0x000002 != 0 {
        cursor += 4;
    }
    if flags & 0x000008 != 0 {
        default_duration = u32_at(bytes, cursor);
        cursor += 4;
    }
    if flags & 0x000010 != 0 {
        default_size = u32_at(bytes, cursor);
    }
    let Some(trun) = child(&fields, *b"trun") else { return };
    let trun_body = traf_base + trun.body;
    let run_flags = u32_at(bytes, trun_body) & 0x00ff_ffff;
    let count = u32_at(bytes, trun_body + 4);
    let mut cursor = trun_body + 8;
    let data_offset = if run_flags & 0x000001 != 0 {
        let value = u32_at(bytes, cursor) as i32;
        cursor += 4;
        value
    } else {
        0
    };
    if run_flags & 0x000004 != 0 {
        cursor += 4;
    }
    // Without `base-data-offset` the samples sit at `moof + data_offset`.
    let mut offset = (base as i64 + i64::from(data_offset)).max(0) as u64;
    for _ in 0..count {
        let mut duration = default_duration;
        let mut size = default_size;
        if run_flags & 0x000100 != 0 {
            duration = u32_at(bytes, cursor);
            cursor += 4;
        }
        if run_flags & 0x000200 != 0 {
            size = u32_at(bytes, cursor);
            cursor += 4;
        }
        if run_flags & 0x000400 != 0 {
            cursor += 4;
        }
        if run_flags & 0x000800 != 0 {
            cursor += 4;
        }
        samples.push(Sample { offset, size, duration });
        offset += u64::from(size);
    }
}

/// Runs of the same duration, which is how `stts` states them.
fn runs(samples: &[Sample]) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::new();
    for sample in samples {
        match out.last_mut() {
            Some((count, delta)) if *delta == sample.duration => *count += 1,
            _ => out.push((1, sample.duration)),
        }
    }
    out
}

fn unity_matrix() -> [u8; 36] {
    let mut matrix = [0_u8; 36];
    for (at, value) in [(0_usize, 0x0001_0000_u32), (16, 0x0001_0000), (32, 0x4000_0000)] {
        matrix[at..at + 4].copy_from_slice(&value.to_be_bytes());
    }
    matrix
}

/// What the new `moov` keeps from the old one.
struct Kept {
    /// `stsd`, `hdlr`, `smhd` and `dinf`, whole and unchanged: the codec setup lives in `stsd`.
    stsd: Vec<u8>,
    hdlr: Vec<u8>,
    smhd: Vec<u8>,
    dinf: Vec<u8>,
    timescale: u32,
    language: [u8; 2],
}

/// Builds a `moov` whose sample table points at `offsets` inside the file's `mdat`.
fn new_moov(kept: &Kept, samples: &[Sample], offsets: &[u64], duration_ms: u64) -> Vec<u8> {
    let total: u64 = samples.iter().map(|s| u64::from(s.duration)).sum();
    let mut mvhd_body = Vec::new();
    mvhd_body.extend_from_slice(&[0_u8; 8]); // creation and modification times
    mvhd_body.extend_from_slice(&1000_u32.to_be_bytes());
    mvhd_body.extend_from_slice(&(duration_ms as u32).to_be_bytes());
    mvhd_body.extend_from_slice(&0x0001_0000_u32.to_be_bytes()); // rate
    mvhd_body.extend_from_slice(&0x0100_u16.to_be_bytes()); // volume
    mvhd_body.extend_from_slice(&[0_u8; 10]);
    mvhd_body.extend_from_slice(&unity_matrix());
    mvhd_body.extend_from_slice(&[0_u8; 24]);
    mvhd_body.extend_from_slice(&2_u32.to_be_bytes()); // next track

    let mut tkhd_body = Vec::new();
    tkhd_body.extend_from_slice(&[0_u8; 8]); // creation and modification times
    tkhd_body.extend_from_slice(&1_u32.to_be_bytes()); // track id
    tkhd_body.extend_from_slice(&0_u32.to_be_bytes());
    tkhd_body.extend_from_slice(&(duration_ms as u32).to_be_bytes());
    tkhd_body.extend_from_slice(&[0_u8; 8]);
    tkhd_body.extend_from_slice(&0_u16.to_be_bytes()); // layer
    tkhd_body.extend_from_slice(&0_u16.to_be_bytes()); // alternate group
    tkhd_body.extend_from_slice(&0x0100_u16.to_be_bytes()); // volume, for a sound track
    tkhd_body.extend_from_slice(&0_u16.to_be_bytes());
    tkhd_body.extend_from_slice(&unity_matrix());
    tkhd_body.extend_from_slice(&0_u32.to_be_bytes()); // width
    tkhd_body.extend_from_slice(&0_u32.to_be_bytes()); // height

    let mut mdhd_body = Vec::new();
    mdhd_body.extend_from_slice(&[0_u8; 8]);
    mdhd_body.extend_from_slice(&kept.timescale.to_be_bytes());
    mdhd_body.extend_from_slice(&(total as u32).to_be_bytes());
    mdhd_body.extend_from_slice(&kept.language);
    mdhd_body.extend_from_slice(&0_u16.to_be_bytes());

    let table = runs(samples);
    let mut stts_body = Vec::new();
    stts_body.extend_from_slice(&(table.len() as u32).to_be_bytes());
    for (count, delta) in &table {
        stts_body.extend_from_slice(&count.to_be_bytes());
        stts_body.extend_from_slice(&delta.to_be_bytes());
    }
    // One sample per chunk keeps the table simple; a player reads it the same either way.
    let mut stsc_body = Vec::new();
    for value in [1_u32, 1, 1] {
        stsc_body.extend_from_slice(&value.to_be_bytes());
    }
    let mut stsz_body = Vec::new();
    stsz_body.extend_from_slice(&0_u32.to_be_bytes());
    stsz_body.extend_from_slice(&(samples.len() as u32).to_be_bytes());
    for sample in samples {
        stsz_body.extend_from_slice(&sample.size.to_be_bytes());
    }
    let mut stco_body = Vec::new();
    stco_body.extend_from_slice(&(offsets.len() as u32).to_be_bytes());
    for offset in offsets {
        stco_body.extend_from_slice(&(*offset as u32).to_be_bytes());
    }

    let mut stbl = Vec::new();
    for part in [
        kept.stsd.clone(),
        full_box(*b"stts", 0, 0, &stts_body),
        full_box(*b"stsc", 0, 0, &stsc_body),
        full_box(*b"stsz", 0, 0, &stsz_body),
        full_box(*b"stco", 0, 0, &stco_body),
    ] {
        stbl.extend_from_slice(&part);
    }
    let mut minf = Vec::new();
    minf.extend_from_slice(&kept.smhd);
    minf.extend_from_slice(&kept.dinf);
    minf.extend_from_slice(&boxed(*b"stbl", &stbl));

    let mut mdia = Vec::new();
    mdia.extend_from_slice(&full_box(*b"mdhd", 0, 0, &mdhd_body));
    mdia.extend_from_slice(&kept.hdlr);
    mdia.extend_from_slice(&boxed(*b"minf", &minf));

    let mut trak = Vec::new();
    trak.extend_from_slice(&full_box(*b"tkhd", 0, 7, &tkhd_body));
    trak.extend_from_slice(&boxed(*b"mdia", &mdia));

    let mut moov = Vec::new();
    moov.extend_from_slice(&full_box(*b"mvhd", 0, 0, &mvhd_body));
    moov.extend_from_slice(&boxed(*b"trak", &trak));
    boxed(*b"moov", &moov)
}

/// Reads what the new `moov` keeps out of the old one.
fn kept_from(bytes: &[u8], moov: &Child) -> io::Result<Kept> {
    let inside = &bytes[moov.body..moov.end];
    let boxes = children(inside);
    let trak = child(&boxes, *b"trak").ok_or_else(|| bad("没有轨道"))?;
    let tk = children(&inside[trak.body..trak.end]);
    let mdia = child(&tk, *b"mdia").ok_or_else(|| bad("没有 mdia"))?;
    // Each level is measured from the body of the box that holds it.
    let md_base = trak.body + mdia.body;
    let md = children(&inside[md_base..trak.body + mdia.end]);
    let mdhd = child(&md, *b"mdhd").ok_or_else(|| bad("没有 mdhd"))?;
    let minf = child(&md, *b"minf").ok_or_else(|| bad("没有 minf"))?;
    let mf_base = md_base + minf.body;
    let mf = children(&inside[mf_base..md_base + minf.end]);
    let stbl = child(&mf, *b"stbl").ok_or_else(|| bad("没有 stbl"))?;
    let stbl_base = mf_base + stbl.body;
    let sb = children(&inside[stbl_base..mf_base + stbl.end]);
    let stsd = child(&sb, *b"stsd").ok_or_else(|| bad("没有 stsd"))?;
    let whole = |parent: usize, found: &Child| inside[parent + found.start..parent + found.end].to_vec();
    let mdhd_body = md_base + mdhd.body;
    let timescale = u32_at(inside, mdhd_body + 12);
    let language = [inside[mdhd_body + 20], inside[mdhd_body + 21]];
    Ok(Kept {
        stsd: whole(stbl_base, stsd),
        hdlr: child(&md, *b"hdlr").map(|c| whole(md_base, c)).ok_or_else(|| bad("没有 hdlr"))?,
        smhd: child(&mf, *b"smhd").map(|c| whole(mf_base, c)).ok_or_else(|| bad("没有 smhd"))?,
        dinf: child(&mf, *b"dinf").map(|c| whole(mf_base, c)).ok_or_else(|| bad("没有 dinf"))?,
        timescale,
        language,
    })
}

fn bad(what: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, what.to_owned())
}

/// Rewrites `source` as a plain M4A at `out`, samples untouched.
pub(crate) fn m4a(source: &Path, out: &Path) -> io::Result<()> {
    let mut file = File::open(source)?;
    let length = file.metadata()?.len();
    let mut start = vec![0_u8; 1 << 20];
    let read = file.read(&mut start)?;
    start.truncate(read);
    let top = children(&start);
    let Some(moov) = child(&top, *b"moov") else { return Err(bad("没有 moov")) };
    let kept = kept_from(&start, moov)?;

    // Every fragment's samples, in the order the file holds them.
    let mut samples = Vec::new();
    let mut at = 0_u64;
    while at < length {
        let mut head = [0_u8; 16];
        file.seek(SeekFrom::Start(at))?;
        if file.read_exact(&mut head).is_err() {
            break;
        }
        let mut size = u32::from_be_bytes(head[0..4].try_into().unwrap()) as u64;
        let kind = [head[4], head[5], head[6], head[7]];
        if size == 1 {
            let mut large = [0_u8; 8];
            file.read_exact(&mut large)?;
            size = u64::from_be_bytes(large);
        } else if size == 0 {
            size = length - at;
        }
        if size < 8 {
            break;
        }
        if &kind == b"moof" {
            // A fragment is small; the samples it points at are not, and stay where they are.
            let mut bytes = vec![0_u8; size as usize];
            file.seek(SeekFrom::Start(at))?;
            file.read_exact(&mut bytes)?;
            read_moof(&bytes, at, &mut samples);
        }
        at += size;
    }
    if samples.is_empty() {
        return Err(bad("没有样本"));
    }

    let total: u64 = samples.iter().map(|s| u64::from(s.duration)).sum();
    let duration_ms = if kept.timescale > 0 { total * 1000 / u64::from(kept.timescale) } else { 0 };
    let ftyp = boxed(*b"ftyp", b"isom\x00\x00\x02\x00isomiso2mp41M4A mp42");
    // The table's length does not depend on the offsets, so it is measured once and filled twice.
    let probe = new_moov(&kept, &samples, &vec![0; samples.len()], duration_ms);
    let mut offsets = Vec::with_capacity(samples.len());
    let mut base = ftyp.len() as u64 + probe.len() as u64 + 8;
    for sample in &samples {
        offsets.push(base);
        base += u64::from(sample.size);
    }
    let moov = new_moov(&kept, &samples, &offsets, duration_ms);

    let mut out_file = BufWriter::new(File::create(out)?);
    out_file.write_all(&ftyp)?;
    out_file.write_all(&moov)?;
    let total_bytes: u64 = samples.iter().map(|s| u64::from(s.size)).sum();
    out_file.write_all(&((total_bytes + 8) as u32).to_be_bytes())?;
    out_file.write_all(b"mdat")?;
    let mut buffer = vec![0_u8; 1 << 20];
    for sample in &samples {
        file.seek(SeekFrom::Start(sample.offset))?;
        let mut left = u64::from(sample.size);
        while left > 0 {
            let take = left.min(buffer.len() as u64) as usize;
            file.read_exact(&mut buffer[..take])?;
            out_file.write_all(&buffer[..take])?;
            left -= take as u64;
        }
    }
    out_file.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ftyp_box() -> Vec<u8> {
        boxed(*b"ftyp", b"iso5   iso5iso6mp41dash")
    }

    /// A `moov` shaped like B站's: one sound track, empty tables, the sample data in fragments.
    fn source_moov() -> Vec<u8> {
        let stsd = full_box(*b"stsd", 0, 0, &{
            let mut body = 1_u32.to_be_bytes().to_vec();
            body.extend_from_slice(&full_box(*b"mp4a", 0, 0, &[0_u8; 8]));
            body
        });
        let stbl = boxed(
            *b"stbl",
            &[
                stsd,
                full_box(*b"stts", 0, 0, &0_u32.to_be_bytes()),
                full_box(*b"stsc", 0, 0, &0_u32.to_be_bytes()),
                full_box(*b"stsz", 0, 0, &[0_u8; 8]),
                full_box(*b"stco", 0, 0, &0_u32.to_be_bytes()),
            ]
            .concat(),
        );
        let minf = boxed(
            *b"minf",
            &[full_box(*b"smhd", 0, 0, &[0_u8; 4]), boxed(*b"dinf", &full_box(*b"dref", 0, 0, &[0_u8; 8])), stbl]
                .concat(),
        );
        let mut mdhd_body = vec![0_u8; 8];
        mdhd_body.extend_from_slice(&1000_u32.to_be_bytes());
        mdhd_body.extend_from_slice(&0_u32.to_be_bytes());
        mdhd_body.extend_from_slice(&[0x55, 0xc4, 0, 0]);
        let mdia = boxed(
            *b"mdia",
            &[full_box(*b"mdhd", 0, 0, &mdhd_body), full_box(*b"hdlr", 0, 0, &[0_u8; 20]), minf].concat(),
        );
        let trak = boxed(*b"trak", &[full_box(*b"tkhd", 0, 7, &[0_u8; 92]), mdia].concat());
        boxed(*b"moov", &[full_box(*b"mvhd", 0, 0, &[0_u8; 96]), trak].concat())
    }

    /// A fragment holding `sizes` as samples of `duration` each.
    fn fragment(sizes: &[u32], duration: u32, data_offset: i32) -> Vec<u8> {
        let mut tfhd_body = 1_u32.to_be_bytes().to_vec();
        tfhd_body.extend_from_slice(&1_u32.to_be_bytes());
        tfhd_body.extend_from_slice(&duration.to_be_bytes());
        tfhd_body.extend_from_slice(&0_u32.to_be_bytes());
        let tfhd = full_box(*b"tfhd", 0, 0x02002a, &tfhd_body);
        let mut trun_body = (sizes.len() as u32).to_be_bytes().to_vec();
        trun_body.extend_from_slice(&data_offset.to_be_bytes());
        for size in sizes {
            trun_body.extend_from_slice(&size.to_be_bytes());
        }
        let trun = full_box(*b"trun", 0, 0x000201, &trun_body);
        let traf = boxed(*b"traf", &[tfhd, trun].concat());
        boxed(*b"moof", &[full_box(*b"mfhd", 0, 0, &1_u32.to_be_bytes()), traf].concat())
    }

    fn source_file() -> Vec<u8> {
        let sizes = [4_u32, 6];
        let mut moof = fragment(&sizes, 512, 0);
        moof = fragment(&sizes, 512, (moof.len() + 8) as i32);
        let mut out = ftyp_box();
        out.extend_from_slice(&source_moov());
        out.extend_from_slice(&moof);
        out.extend_from_slice(&(8_u32 + 10).to_be_bytes());
        out.extend_from_slice(b"mdat");
        out.extend_from_slice(&[1, 2, 3, 4]);
        out.extend_from_slice(&[5, 6, 7, 8, 9, 10]);
        out
    }

    fn found(bytes: &[u8], kind: [u8; 4]) -> Option<usize> {
        bytes.windows(4).position(|window| window == kind)
    }

    #[test]
    #[ignore]
    fn real_file() {
        let src = std::path::Path::new(
            "C:/Users/Administrator/AppData/Local/Packages/Rubi_acy06xm5xb8wg/LocalState/downloads/BV1gb8h6SEFT/audio.m4s",
        );
        let dst = std::path::Path::new("C:/Users/Administrator/AppData/Local/hermes/cache/scratch/rust2.m4a");
        m4a(src, dst).unwrap();
        println!("wrote {}", dst.display());
    }

    #[test]
    fn a_fragment_only_file_becomes_a_plain_one() {
        let dir = std::env::temp_dir().join("rubi-remux-test");
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("in.m4s");
        let out = dir.join("out.m4a");
        std::fs::write(&source, source_file()).unwrap();
        m4a(&source, &out).unwrap();
        let bytes = std::fs::read(&out).unwrap();

        let moov = found(&bytes, *b"moov").expect("no moov in the result");
        assert!(moov < found(&bytes, *b"mdat").unwrap(), "moov comes before mdat");
        let stsz = found(&bytes, *b"stsz").expect("no stsz");
        assert_eq!(u32_at(&bytes, stsz + 12), 2, "both samples are in the table");
        let stco = found(&bytes, *b"stco").expect("no stco");
        assert_eq!(u32_at(&bytes, stco + 8), 2, "one chunk per sample");
        let mdhd = found(&bytes, *b"mdhd").expect("no mdhd");
        assert_eq!(u32_at(&bytes, mdhd + 16), 1000, "the source's timescale is kept");
        assert_eq!(u32_at(&bytes, mdhd + 20), 1024, "2 x 512, at that timescale");
        assert_eq!(u32_at(&bytes, found(&bytes, *b"mvhd").unwrap() + 20), 1024, "1024 ticks of a 1000 timescale");
        assert!(!bytes.windows(4).any(|w| w == b"moof"), "no fragments are left");
        assert_eq!(&bytes[bytes.len() - 10..], &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], "the samples came along");
    }
}
