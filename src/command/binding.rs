use super::Command;

/// Fixed Rust targets for context maps. Ordinary command identities are exactly
/// those used by M-x; prompt-only actions belong to the existing input owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binding {
    Command(Command),
    Prompt(PromptAction),
}

/// Stage-dependent operations already handled by focused prompts. These names
/// are context-only catalog entries, not additional editor/M-x commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptAction {
    Accept,
    Complete,
    Previous,
    Next,
}

impl PromptAction {
    pub const ALL: [Self; 4] = [Self::Accept, Self::Complete, Self::Previous, Self::Next];

    pub fn name(self) -> &'static str {
        match self {
            Self::Accept => "minibuffer-accept",
            Self::Complete => "minibuffer-complete",
            Self::Previous => "minibuffer-previous",
            Self::Next => "minibuffer-next",
        }
    }
}

impl Binding {
    pub fn named(name: &str) -> Option<Self> {
        Command::named(name).map(Self::Command).or_else(|| {
            PromptAction::ALL
                .into_iter()
                .find(|action| action.name() == name)
                .map(Self::Prompt)
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Command(command) => command.name(),
            Self::Prompt(action) => action.name(),
        }
    }
}
