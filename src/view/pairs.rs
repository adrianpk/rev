use crate::config::{PairHighlight, Pairs};

/// Runtime overrides and remembered style belong to one view. The buffer only
/// supplies resolved startup defaults, including after a language change.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PairPresentation {
    style: Option<PairHighlight>,
    last_non_off: Option<PairHighlight>,
}

impl PairPresentation {
    pub fn style(self, defaults: Pairs) -> PairHighlight {
        self.style.unwrap_or(defaults.highlight)
    }

    pub fn last_non_off(self, defaults: Pairs) -> PairHighlight {
        self.last_non_off.unwrap_or(match defaults.highlight {
            PairHighlight::Off => PairHighlight::Depth,
            style => style,
        })
    }

    pub fn choose(&mut self, style: PairHighlight, defaults: Pairs) {
        let remembered = if style == PairHighlight::Off {
            self.style(defaults)
        } else {
            style
        };
        if remembered != PairHighlight::Off {
            self.last_non_off = Some(remembered);
        }
        self.style = Some(style);
    }

    pub fn toggle(&mut self, defaults: Pairs) {
        let style = self.style(defaults);
        if style == PairHighlight::Off {
            self.choose(self.last_non_off(defaults), defaults);
        } else {
            self.last_non_off = Some(style);
            self.style = Some(PairHighlight::Off);
        }
    }
}
