use crate::buffer::BufferId;
use crate::config::InteractionProfile;
use crate::window::ViewId;

use super::{Focus, Input};

/// Transient state of the selected startup profile, never persisted to settings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditingState {
    #[default]
    Normal,
    Insert,
    Replace,
    VisualCharacter,
    VisualLine,
    VisualBlock,
}

impl EditingState {
    pub(crate) fn visual(self) -> Option<crate::buffer::Visual> {
        use crate::buffer::Visual;
        match self {
            Self::VisualCharacter => Some(Visual::Character),
            Self::VisualLine => Some(Visual::Line),
            Self::VisualBlock => Some(Visual::Block),
            _ => None,
        }
    }

    pub(crate) fn from_visual(shape: crate::buffer::Visual) -> Self {
        match shape {
            crate::buffer::Visual::Character => Self::VisualCharacter,
            crate::buffer::Visual::Line => Self::VisualLine,
            crate::buffer::Visual::Block => Self::VisualBlock,
        }
    }

    pub(crate) fn text_entry(self) -> bool {
        matches!(self, Self::Insert | Self::Replace)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Replace => "REPLACE",
            Self::VisualCharacter => "VISUAL",
            Self::VisualLine => "VISUAL LINE",
            Self::VisualBlock => "VISUAL BLOCK",
        }
    }
}

impl Input {
    pub fn interaction_profile(&self) -> InteractionProfile {
        self.profile
    }

    pub fn editing_state(&self) -> Option<EditingState> {
        (self.profile == InteractionProfile::Modal).then_some(self.editing_state)
    }

    pub(crate) fn set_editing_state(&mut self, state: EditingState) {
        if self.editing_state != state {
            self.retire_command_hints();
        }
        self.clear_prefix();
        self.suppress_repeat = true;
        self.editing_state = state;
    }

    /// A changed native view/buffer cannot inherit a text-entry continuation.
    pub(crate) fn editing_owner(&mut self, view: ViewId, buffer: BufferId) {
        let owner = Some((view, buffer));
        if self.editing_owner != owner {
            self.retire_command_hints();
            if self.profile == InteractionProfile::Modal {
                self.set_editing_state(EditingState::Normal);
            }
            self.editing_owner = owner;
        }
    }

    pub(super) fn modal_editing(&self) -> bool {
        self.profile == InteractionProfile::Modal
            && self.focus() == Focus::Editing
            && !self.workspace_startup
            && !self.workspace_tree
            && !self.workspace_bookmarks
            && !self.workspace_git
            && !self.workspace_git_commit
            && !matches!(
                self.workspace_org,
                Some(super::OrgContext::Agenda | super::OrgContext::Calendar)
            )
    }

    pub(super) fn modal_context(&self) -> super::keymap::Context {
        match self.editing_state {
            EditingState::Normal => super::keymap::Context::VimNormal,
            EditingState::Insert => super::keymap::Context::VimInsert,
            EditingState::Replace => super::keymap::Context::VimReplace,
            EditingState::VisualCharacter => super::keymap::Context::VimVisualCharacter,
            EditingState::VisualLine => super::keymap::Context::VimVisualLine,
            EditingState::VisualBlock => super::keymap::Context::VimVisualBlock,
        }
    }

    pub(super) fn active_map(&self) -> &super::keymap::Map {
        if self.modal_editing() {
            let local = self.workspace_org.map(super::OrgContext::keymap);
            self.keymaps.vim_map(self.modal_context(), local)
        } else {
            self.keymaps.map(self.map_context())
        }
    }
}
