use super::Input;
use super::keymap::Context;

/// A host-owned Org workspace role, below transient prompts and tree focus.
/// Selecting a role routes input; it does not create agenda, draft or job state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrgContext {
    Editing,
    Agenda,
    Capture,
    Calendar,
    SourceEdit,
}

impl OrgContext {
    pub const ALL: [Self; 5] = [
        Self::Editing,
        Self::Agenda,
        Self::Capture,
        Self::Calendar,
        Self::SourceEdit,
    ];

    pub fn keymap(self) -> Context {
        match self {
            Self::Editing => Context::Org,
            Self::Agenda => Context::OrgAgenda,
            Self::Capture => Context::OrgCapture,
            Self::Calendar => Context::OrgCalendar,
            Self::SourceEdit => Context::OrgSourceEdit,
        }
    }

    fn accepts_text(self) -> bool {
        !matches!(self, Self::Agenda | Self::Calendar)
    }
}

impl Input {
    /// Switching workspace ownership retires pending chords and Escape timing,
    /// while an active prompt keeps its draft and precedence until it exits.
    pub fn set_org_context(&mut self, context: Option<OrgContext>) {
        if self.workspace_org != context {
            self.clear_prefix();
            self.suppress_repeat = true;
            self.workspace_org = context;
        }
    }

    pub fn org_context(&self) -> Option<OrgContext> {
        self.workspace_org
    }

    pub(super) fn accepts_text(&self) -> bool {
        self.workspace_org.is_none_or(OrgContext::accepts_text)
    }
}
