#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionCommand {
    Set,
    Exchange,
}

impl RegionCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Set => "set-mark-command",
            Self::Exchange => "exchange-point-and-mark",
        }
    }
}
