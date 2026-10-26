#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeCommand {
    Toggle,
    Focus,
}

impl TreeCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Toggle => "toggle-file-tree",
            Self::Focus => "focus-file-tree",
        }
    }
}
