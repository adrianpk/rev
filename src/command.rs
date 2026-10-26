/// Only commands with a working handler belong in the public registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    KeyboardQuit,
}

impl Command {
    pub const ALL: [Self; 2] = [Self::Quit, Self::KeyboardQuit];

    pub fn name(self) -> &'static str {
        match self {
            Self::Quit => "kill-terminal",
            Self::KeyboardQuit => "keyboard-quit",
        }
    }

    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|command| command.name() == name)
    }
}
