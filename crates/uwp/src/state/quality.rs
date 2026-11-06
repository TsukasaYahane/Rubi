//! What a player can play at.

/// What a player offers: every (qn, name), and the qn playing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Offer {
    pub options: Vec<(u32, String)>,
    pub current: u32,
}

impl Offer {
    /// What the button says.
    pub fn label(&self) -> &str {
        self.options.iter().find(|(qn, _)| *qn == self.current).map_or("清晰度", |(_, name)| name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_label_is_the_name_of_what_plays() {
        let offer = Offer { options: vec![(80, "1080P".into()), (64, "720P".into())], current: 64 };
        assert_eq!(offer.label(), "720P");
        assert_eq!(Offer { current: 16, ..offer }.label(), "清晰度");
    }
}
