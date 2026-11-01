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
}
