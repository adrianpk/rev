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

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Kill => "Remove the active linear region into the ring.",
            Self::Copy => "Copy the active linear region.",
            Self::Yank => "Insert the newest linear ring entry.",
            Self::Pop => "Rotate the current yank's exact insertion range.",
        }
    }
}
