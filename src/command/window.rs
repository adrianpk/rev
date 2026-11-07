#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowCommand {
    Delete,
    Keep,
    Other,
    Focus(crate::window::Direction),
    Balance,
}

impl WindowCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Delete => "delete-window",
            Self::Keep => "delete-other-windows",
            Self::Other => "other-window",
            Self::Balance => "balance-windows",
            Self::Focus(direction) => match direction {
                crate::window::Direction::Left => "windmove-left",
                crate::window::Direction::Down => "windmove-down",
                crate::window::Direction::Up => "windmove-up",
                crate::window::Direction::Right => "windmove-right",
            },
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Delete => "Close the selected view, retaining its buffer.",
            Self::Keep => "Keep only the selected view.",
            Self::Other => "Cycle editing views.",
            Self::Balance => {
                "Equalize existing pane divisions, preserving focus and startup split policy."
            }
            Self::Focus(direction) => match direction {
                crate::window::Direction::Left => "Focus the nearest overlapping pane to the left.",
                crate::window::Direction::Down => "Focus the nearest overlapping pane below.",
                crate::window::Direction::Up => "Focus the nearest overlapping pane above.",
                crate::window::Direction::Right => {
                    "Focus the nearest overlapping pane to the right."
                }
            },
        }
    }
}
