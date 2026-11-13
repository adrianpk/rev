use std::fmt;

use crate::config::Snapshot;

use super::Buffer;

/// Explicit native major-mode choices. Automatic visiting-language resolution
/// remains the default until the user selects a mode for this canonical buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Text,
    Org,
    Markdown,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Text => "text-mode",
            Self::Org => "org-mode",
            Self::Markdown => "markdown-mode",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Text => "Select ordinary text editing.",
            Self::Org => "Select native Org for the current buffer.",
            Self::Markdown => "Select native Markdown for the current buffer.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModeError {
    Disabled,
    MarkdownDisabled,
    Transient,
}

impl fmt::Display for ModeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Disabled => "Org mode is disabled by org.enabled",
            Self::MarkdownDisabled => "Markdown mode is disabled by markdown.enabled",
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
        if mode == Mode::Markdown && !self.markdown.enabled {
            return Err(ModeError::MarkdownDisabled);
        }
        self.mode_choice = Some(mode);
        self.activate_org(mode == Mode::Org);
        self.activate_markdown(mode == Mode::Markdown);
        self.refresh_syntax();
        Ok(())
    }

    pub(super) fn apply_mode(&mut self, settings: &Snapshot) {
        self.org_enabled = settings.settings().org.enabled;
        self.apply_markdown_mode(settings);
        let requested = self.mode_choice.map_or_else(
            || self.language(settings) == "org",
            |mode| mode == Mode::Org,
        );
        let admitted = !self.transient || self.is_readonly() && self.mode_choice == Some(Mode::Org);
        self.activate_org(admitted && self.org_enabled && requested);
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

impl Buffer {
    fn apply_markdown_mode(&mut self, settings: &Snapshot) {
        self.markdown.enabled = settings.settings().markdown.enabled;
        let requested = self.mode_choice.map_or_else(
            || self.language(settings) == "markdown",
            |mode| mode == Mode::Markdown,
        );
        self.activate_markdown(!self.transient && self.markdown.enabled && requested);
    }
}
