use crate::buffer::BufferId;
use crate::command::{Command, ModalCommand};
use crate::config::InteractionProfile;
use crate::window::ViewId;

use super::{Focus, Input, Key, Outcome};

/// Transient state of the selected startup profile, never persisted to settings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditingState {
    #[default]
    Normal,
    Insert,
}

impl Input {
    pub fn interaction_profile(&self) -> InteractionProfile {
        self.profile
    }

    pub fn editing_state(&self) -> Option<EditingState> {
        (self.profile == InteractionProfile::Vim).then_some(self.editing_state)
    }

    pub(crate) fn set_editing_state(&mut self, state: EditingState) {
        self.clear_prefix();
        self.suppress_repeat = true;
        self.editing_state = state;
    }

    /// A changed native view/buffer cannot inherit a text-entry continuation.
    pub(crate) fn editing_owner(&mut self, view: ViewId, buffer: BufferId) {
        let owner = Some((view, buffer));
        if self.editing_owner != owner {
            self.set_editing_state(EditingState::Normal);
            self.editing_owner = owner;
        }
    }

    pub(super) fn modal_editing(&self) -> bool {
        self.profile == InteractionProfile::Vim
            && self.focus() == Focus::Editing
            && !self.workspace_tree
            && !self.workspace_bookmarks
            && !matches!(
                self.workspace_org,
                Some(super::OrgContext::Agenda | super::OrgContext::Calendar)
            )
    }

    pub(super) fn modal_key(&self, key: Key) -> Option<Outcome> {
        if !self.modal_editing() || self.editing_state != EditingState::Normal {
            return None;
        }
        let command = match key.plain()? {
            crossterm::event::KeyCode::Char('i') => ModalCommand::Insert,
            crossterm::event::KeyCode::Char('a') => ModalCommand::InsertAfter,
            crossterm::event::KeyCode::Char('I') => ModalCommand::InsertLineFirst,
            crossterm::event::KeyCode::Char('A') => ModalCommand::InsertLineEnd,
            crossterm::event::KeyCode::Char('o') => ModalCommand::OpenBelow,
            crossterm::event::KeyCode::Char('O') => ModalCommand::OpenAbove,
            _ => return Some(Outcome::Message("Unbound Normal key")),
        };
        Some(Outcome::Command(Command::Modal(command)))
    }
}
