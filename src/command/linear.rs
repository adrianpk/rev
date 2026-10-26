#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinearCommand {
    Kill,
    Copy,
    Yank,
    Pop,
}

impl LinearCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Kill => "kill-region",
            Self::Copy => "kill-ring-save",
            Self::Yank => "yank",
            Self::Pop => "yank-pop",
        }
    }
}
