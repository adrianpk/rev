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

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Set => "Activate a linear region at point.",
            Self::Exchange => "Exchange point and mark.",
        }
    }
}
