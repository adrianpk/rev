mod buffer;
mod motion;
mod scroll;
mod split;
mod window;

pub use crate::buffer::edit::History;
pub use crate::window::Split;
pub use buffer::BufferCommand;
pub use motion::Motion;
pub use scroll::Scroll;
pub use window::WindowCommand;

/// Only commands with a working handler belong in the public registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    FindFile,
    SaveBuffer,
    WriteFile,
    KeyboardQuit,
    Newline,
    InsertTab,
    BackwardDeleteChar,
    DeleteChar,
    Motion(Motion),
    Scroll(Scroll),
    History(History),
    Buffer(BufferCommand),
    Split(Split),
    Window(WindowCommand),
}

impl Command {
    pub const ALL: [Self; 32] = [
        Self::Quit,
        Self::FindFile,
        Self::SaveBuffer,
        Self::WriteFile,
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
        Self::Scroll(Scroll::NextLine),
        Self::Scroll(Scroll::PreviousLine),
        Self::Scroll(Scroll::PageForward),
        Self::Scroll(Scroll::PageBackward),
        Self::Scroll(Scroll::Recenter),
        Self::History(History::Undo),
        Self::History(History::Redo),
        Self::Buffer(BufferCommand::Switch),
        Self::Buffer(BufferCommand::List),
        Self::Buffer(BufferCommand::Kill),
        Self::Split(Split::Below),
        Self::Split(Split::Right),
        Self::Window(WindowCommand::Delete),
        Self::Window(WindowCommand::Keep),
        Self::Window(WindowCommand::Other),
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Motion(motion) => motion.name(),
            Self::Scroll(scroll) => scroll.name(),
            Self::History(history) => history.name(),
            Self::Buffer(command) => command.name(),
            Self::Split(split) => split.name(),
            Self::Window(command) => command.name(),
            _ => self.action_name(),
        }
    }

    fn action_name(self) -> &'static str {
        match self {
            Self::Quit => "kill-terminal",
            Self::FindFile => "find-file",
            Self::SaveBuffer => "save-buffer",
            Self::WriteFile => "write-file",
            Self::KeyboardQuit => "keyboard-quit",
            Self::Newline => "newline",
            Self::InsertTab => "insert-tab",
            Self::BackwardDeleteChar => "backward-delete-char",
            Self::DeleteChar => "delete-char",
            _ => unreachable!("navigation commands delegate their names before action lookup"),
        }
    }

    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|command| command.name() == name)
    }
}
