use super::Split;

impl Split {
    pub fn name(self) -> &'static str {
        match self {
            Self::Below => "split-window-below",
            Self::Right => "split-window-right",
        }
    }
}
