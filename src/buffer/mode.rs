use std::fmt;

use crate::config::Snapshot;

use super::Buffer;

/// Explicit native major-mode choices. Automatic visiting-language resolution
/// remains the default until the user selects a mode for this canonical buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Text,
    Org,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Text => "text-mode",
            Self::Org => "org-mode",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeError {
    Disabled,
    Transient,
}

impl fmt::Display for ModeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Disabled => "Org mode is disabled by org.enabled",
            Self::Transient => "Capture draft retains its transient editing context",
        })
    }
}

impl std::error::Error for ModeError {}

impl Buffer {
    pub fn is_org(&self) -> bool {
        self.org_active
    }

    /// Mode changes affect derived behavior only. All views share this choice;
    /// canonical text, read-only state, revision, saved state and history survive.
    pub fn select_mode(&mut self, mode: Mode) -> Result<(), ModeError> {
        if self.transient {
            return Err(ModeError::Transient);
        }
        if mode == Mode::Org && !self.org_enabled {
            return Err(ModeError::Disabled);
        }
        self.mode_choice = Some(mode);
        self.activate_org(mode == Mode::Org);
        self.refresh_syntax();
        Ok(())
    }

    pub(super) fn apply_mode(&mut self, settings: &Snapshot) {
        self.org_enabled = settings.settings().org.enabled;
        let requested = self.mode_choice.map_or_else(
            || self.language(settings) == "org",
            |mode| mode == Mode::Org,
        );
        self.activate_org(!self.transient && self.org_enabled && requested);
    }

    fn activate_org(&mut self, active: bool) {
        self.org_active = active;
        if !active {
            self.org = None;
            self.org_highlights = None;
            self.org_outline = None;
            self.org_attempt = None;
        }
    }
}
