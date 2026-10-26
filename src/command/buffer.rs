#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferCommand {
    Switch,
    List,
    Kill,
}

impl BufferCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Switch => "switch-to-buffer",
            Self::List => "list-buffers",
            Self::Kill => "kill-buffer",
        }
    }
}
