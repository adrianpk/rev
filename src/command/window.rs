#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowCommand {
    Delete,
    Keep,
    Other,
}

impl WindowCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Delete => "delete-window",
            Self::Keep => "delete-other-windows",
            Self::Other => "other-window",
        }
    }
}
