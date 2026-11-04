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
    OpenHere,
    OpenOtherPane,
    SendNextKey,
}

impl TerminalCommand {
    pub fn name(self) -> &'static str {
        if self == Self::SendNextKey {
            return "terminal-send-next-key";
        }
        self.placement_name()
    }

    fn placement_name(self) -> &'static str {
        match self {
            Self::Open => "terminal-open",
            Self::Toggle => "terminal-toggle",
            Self::NewBuffer => "terminal-new-buffer",
            Self::MoveToBuffer => "terminal-move-to-buffer",
            Self::MoveToPanel => "terminal-move-to-panel",
            Self::New => "terminal-new",
            Self::List => "terminal-list",
            Self::Rename => "terminal-rename",
            Self::OpenHere | Self::OpenOtherPane => self.entry_name(),
            Self::SendNextKey => unreachable!("input commands named separately"),
        }
    }

    fn entry_name(self) -> &'static str {
        match self {
            Self::OpenHere => "terminal-open-here",
            Self::OpenOtherPane => "terminal-open-other-pane",
            _ => unreachable!("directory entry commands only"),
        }
    }
}
