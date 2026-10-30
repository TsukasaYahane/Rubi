//! Dependency-free title tokenisation: lower-cased ASCII words plus CJK bigrams.

pub fn tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    let mut cjk: Vec<char> = Vec::new();

    let flush_word = |word: &mut String, out: &mut Vec<String>| {
        if word.chars().count() >= 2 {
            out.push(word.to_lowercase());
        }
        word.clear();
    };
    let flush_cjk = |cjk: &mut Vec<char>, out: &mut Vec<String>| {
        out.extend(cjk.windows(2).map(|w| w.iter().collect::<String>()));
        cjk.clear();
    };

    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            flush_cjk(&mut cjk, &mut out);
            word.push(c);
        } else if is_cjk(c) {
            flush_word(&mut word, &mut out);
            cjk.push(c);
        } else {
            flush_word(&mut word, &mut out);
            flush_cjk(&mut cjk, &mut out);
        }
    }
    flush_word(&mut word, &mut out);
    flush_cjk(&mut cjk, &mut out);
    out.sort();
    out.dedup();
    out
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x3040..=0x30FF)
}

#[cfg(test)]
mod tests {
    #[test]
    fn mixes_words_and_bigrams() {
        assert_eq!(super::tokens("Rust 异步运行时"), vec!["rust", "异步", "步运", "行时", "运行"]);
    }
}
