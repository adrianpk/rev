mod motion;

pub use motion::Motion;

/// Only commands with a working handler belong in the public registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    KeyboardQuit,
    Newline,
    InsertTab,
    BackwardDeleteChar,
    DeleteChar,
    Motion(Motion),
}

impl Command {
    pub const ALL: [Self; 14] = [
        Self::Quit,
        Self::KeyboardQuit,
        Self::Newline,
        Self::InsertTab,
        Self::BackwardDeleteChar,
        Self::DeleteChar,
        Self::Motion(Motion::ForwardChar),
        Self::Motion(Motion::BackwardChar),
        Self::Motion(Motion::LineStart),
        Self::Motion(Motion::LineEnd),
        Self::Motion(Motion::BufferStart),
        Self::Motion(Motion::BufferEnd),
        Self::Motion(Motion::ForwardWord),
        Self::Motion(Motion::BackwardWord),
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Quit => "kill-terminal",
            Self::KeyboardQuit => "keyboard-quit",
            Self::Newline => "newline",
            Self::InsertTab => "insert-tab",
            Self::BackwardDeleteChar => "backward-delete-char",
            Self::DeleteChar => "delete-char",
            Self::Motion(motion) => motion.name(),
        }
    }

    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|command| command.name() == name)
    }
}
