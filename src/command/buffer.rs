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

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Switch => "Select a live buffer by name.",
            Self::List => "Select from the live buffer inventory.",
            Self::Kill => "Close a buffer with modified-text protection.",
        }
    }
}
