/// Only commands with a working handler belong in the public registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    KeyboardQuit,
    Newline,
    InsertTab,
    BackwardDeleteChar,
    DeleteChar,
}

impl Command {
    pub const ALL: [Self; 6] = [
        Self::Quit,
        Self::KeyboardQuit,
        Self::Newline,
        Self::InsertTab,
        Self::BackwardDeleteChar,
        Self::DeleteChar,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Quit => "kill-terminal",
            Self::KeyboardQuit => "keyboard-quit",
            Self::Newline => "newline",
            Self::InsertTab => "insert-tab",
            Self::BackwardDeleteChar => "backward-delete-char",
            Self::DeleteChar => "delete-char",
        }
    }

    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|command| command.name() == name)
    }
}
