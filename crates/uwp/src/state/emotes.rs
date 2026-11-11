//! Text with B站 emote tokens (`[doge]`) cut into plain runs and emotes with a known image, the
//! same tokens Android draws. No XAML here.

/// A run of text, or an emote token with its image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece<'a> {
    Text(&'a str),
    Emote { token: &'a str, url: &'a str },
}

/// Longest token taken for an emote: decoration emotes are named like `[香奈美·追寻那道光 应援装扮_啊]`.
const LONGEST: usize = 40;

/// Cuts `text` at the tokens `lookup` knows an image for; the rest stays text.
pub fn pieces<'a>(text: &'a str, lookup: impl Fn(&str) -> Option<&'a str>) -> Vec<Piece<'a>> {
    let mut out = Vec::new();
    let mut plain = 0;
    let mut at = 0;
    while let Some(open) = text[at..].find('[').map(|i| at + i) {
        let close = text[open + 1..].find([']', '[', '\n']).map(|i| open + 1 + i);
        let Some(close) = close.filter(|c| text.as_bytes()[*c] == b']') else {
            at = open + 1;
            continue;
        };
        let token = &text[open..=close];
        let inner = token.chars().count() - 2;
        match lookup(token).filter(|_| (1..=LONGEST).contains(&inner)) {
            Some(url) => {
                if plain < open {
                    out.push(Piece::Text(&text[plain..open]));
                }
                out.push(Piece::Emote { token, url });
                plain = close + 1;
                at = close + 1;
            }
            None => at = open + 1,
        }
    }
    if plain < text.len() {
        out.push(Piece::Text(&text[plain..]));
    }
    out
}

/// Whether `text` has any emote `lookup` knows.
pub fn has_emotes<'a>(text: &'a str, lookup: impl Fn(&str) -> Option<&'a str>) -> bool {
    text.contains('[') && pieces(text, lookup).iter().any(|p| matches!(p, Piece::Emote { .. }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known(token: &str) -> Option<&'static str> {
        match token {
            "[doge]" => Some("https://e/doge.png"),
            "[tv_笑]" => Some("https://e/tv.png"),
            _ => None,
        }
    }

    #[test]
    fn known_tokens_become_emotes() {
        assert_eq!(
            pieces("好[doge]呀[tv_笑]", known),
            [
                Piece::Text("好"),
                Piece::Emote { token: "[doge]", url: "https://e/doge.png" },
                Piece::Text("呀"),
                Piece::Emote { token: "[tv_笑]", url: "https://e/tv.png" },
            ]
        );
    }

    #[test]
    fn unknown_and_broken_tokens_stay_text() {
        assert_eq!(pieces("[不认识][doge", known), [Piece::Text("[不认识][doge")]);
        assert_eq!(
            pieces("[[doge]]", known),
            [Piece::Text("["), Piece::Emote { token: "[doge]", url: "https://e/doge.png" }, Piece::Text("]")]
        );
        assert_eq!(pieces("", known), []);
        assert!(!has_emotes("没有表情", known));
        assert!(has_emotes("[doge]", known));
    }
}
