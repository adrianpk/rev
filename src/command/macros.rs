/// Native recording controls. Later macro surfaces have separate delivery units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacroCommand {
    StartOrCounter,
    Start,
    End,
    EndOrCall,
    EndAndCall,
}

impl MacroCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::StartOrCounter => "kmacro-start-macro-or-insert-counter",
            Self::Start => "kmacro-start-macro",
            Self::End => "kmacro-end-macro",
            Self::EndOrCall => "kmacro-end-or-call-macro",
            Self::EndAndCall => "kmacro-end-and-call-macro",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::StartOrCounter => "Start recording, or insert the current macro counter.",
            Self::Start => "Start or append to a keyboard macro.",
            Self::End => "Finish recording a keyboard macro.",
            Self::EndOrCall => "Finish recording, or run the current keyboard macro.",
            Self::EndAndCall => "Finish recording and run the current keyboard macro.",
        }
    }
}
