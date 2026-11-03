#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalCommand {
    Open,
    Toggle,
}

impl TerminalCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Open => "terminal-open",
            Self::Toggle => "terminal-toggle",
        }
    }
}
