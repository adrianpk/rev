#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalCommand {
    Open,
    Toggle,
    NewBuffer,
    MoveToBuffer,
    MoveToPanel,
    New,
    List,
    Rename,
}

impl TerminalCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Open => "terminal-open",
            Self::Toggle => "terminal-toggle",
            Self::NewBuffer => "terminal-new-buffer",
            Self::MoveToBuffer => "terminal-move-to-buffer",
            Self::MoveToPanel => "terminal-move-to-panel",
            Self::New => "terminal-new",
            Self::List => "terminal-list",
            Self::Rename => "terminal-rename",
        }
    }
}
