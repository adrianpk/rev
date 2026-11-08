/// Native keyboard macro controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MacroCommand {
    StartOrCounter,
    Start,
    End,
    EndOrCall,
    EndAndCall,
    CallRepeat,
    CallSecond,
    Next,
    Previous,
    Delete,
    Swap,
    Redisplay,
    InsertCounter,
    SetCounter,
    AddCounter,
    SetFormat,
    ApplyLines,
    Query,
    Resume,
    Abort,
    Name,
    Bind,
}

impl MacroCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::StartOrCounter => "kmacro-start-macro-or-insert-counter",
            Self::Start => "kmacro-start-macro",
            Self::End => "kmacro-end-macro",
            Self::EndOrCall => "kmacro-end-or-call-macro",
            Self::EndAndCall => "kmacro-end-and-call-macro",
            Self::CallRepeat => "kmacro-end-or-call-macro-repeat",
            Self::CallSecond => "kmacro-call-ring-2nd-repeat",
            Self::Next => "kmacro-cycle-ring-next",
            Self::Previous => "kmacro-cycle-ring-previous",
            Self::Delete => "kmacro-delete-ring-head",
            Self::Swap => "kmacro-swap-ring",
            Self::Redisplay => "kmacro-redisplay",
            Self::InsertCounter => "kmacro-insert-counter",
            Self::SetCounter => "kmacro-set-counter",
            Self::AddCounter => "kmacro-add-counter",
            Self::SetFormat => "kmacro-set-format",
            Self::ApplyLines => "apply-macro-to-region-lines",
            Self::Query => "kbd-macro-query",
            Self::Resume => "exit-recursive-edit",
            Self::Abort => "abort-recursive-edit",
            Self::Name => "kmacro-name-last-macro",
            Self::Bind => "kmacro-bind-to-key",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::StartOrCounter => "Start recording, or insert the current macro counter.",
            Self::Start => "Start or append to a keyboard macro.",
            Self::End => "Finish recording a keyboard macro.",
            Self::EndOrCall => "Finish recording, or run the current keyboard macro.",
            Self::EndAndCall => "Finish recording and run the current keyboard macro.",
            Self::CallRepeat => "Run the current macro and enable ring repeat keys.",
            Self::CallSecond => "Run the second macro and enable ring repeat keys.",
            Self::Next => "Select the next older keyboard macro.",
            Self::Previous => "Select the next newer keyboard macro.",
            Self::Delete => "Remove the current macro from the session ring.",
            Self::Swap => "Exchange the first two macros in the ring.",
            Self::Redisplay => "Refresh the display during macro playback.",
            Self::InsertCounter => "Insert a macro counter value.",
            Self::SetCounter => "Set or reset the current macro counter.",
            Self::AddCounter => "Add to or restore the current macro counter.",
            Self::SetFormat => "Set the integer format for macro counters.",
            Self::ApplyLines => "Run the current macro on logical lines in the region.",
            Self::Query => "Add a confirmation or editing pause to a macro.",
            Self::Resume => "Resume a macro after its manual editing pause.",
            Self::Abort => "Abort a macro editing pause and its playback.",
            Self::Name => "Give the current macro a session command name.",
            Self::Bind => "Bind a session macro to a key sequence.",
        }
    }
}
