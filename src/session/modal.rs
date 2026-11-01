use crate::command::ModalCommand;
use crate::config::InteractionProfile;
use crate::coordinates::{Coordinates, Scalar};
use crate::input::{EditingState, Focus};
use crate::view::Error;

use super::{Action, Session};

impl Session {
    pub fn interaction_profile(&self) -> InteractionProfile {
        self.input.interaction_profile()
    }

    pub fn editing_state(&self) -> Option<EditingState> {
        self.input.editing_state()
    }

    pub(super) fn modal_command(&mut self, command: ModalCommand) -> Action {
        if self.interaction_profile() != InteractionProfile::Vim {
            return self.message("Modal state requires the Vim startup profile");
        }
        if self.input.focus() != Focus::Editing
            || self.tree.focused()
            || self.bookmark_list_active()
            || matches!(
                self.input.org_context(),
                Some(crate::input::OrgContext::Agenda | crate::input::OrgContext::Calendar)
            )
        {
            return self.message("A component owns input focus");
        }
        match self.enter_modal(command) {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn enter_modal(&mut self, command: ModalCommand) -> Result<(), Error> {
        let target = self.modal_target(command)?;
        if matches!(command, ModalCommand::OpenBelow | ModalCommand::OpenAbove) {
            let newline = self.buffer()?.newline();
            let cursor = if command == ModalCommand::OpenAbove {
                target
            } else {
                Scalar(
                    target
                        .0
                        .checked_add(newline.chars().count())
                        .ok_or(crate::buffer::edit::Error::Capacity)?,
                )
            };
            self.replace_at(target, newline, cursor)?;
        } else if target != self.view().point.position {
            let mut point = self.view().point;
            point.position = target;
            self.place_point(point)?;
        }
        self.input
            .set_editing_state(if command == ModalCommand::Normal {
                EditingState::Normal
            } else {
                EditingState::Insert
            });
        Ok(())
    }

    /// Odra leaves the native point in place on Insert exit. Do not decrement
    /// a byte or synthesize a separate Normal cursor.
    pub(super) fn modal_target(&self, command: ModalCommand) -> Result<Scalar, Error> {
        let point = self.view().point.position;
        let buffer = self.buffer()?;
        self.view().check_point(buffer)?;
        let coordinates = Coordinates::new(buffer)?;
        Ok(match command {
            ModalCommand::Normal | ModalCommand::Insert => point,
            ModalCommand::InsertAfter => coordinates
                .next_observed(point)?
                .min(coordinates.line_end(point)?),
            ModalCommand::InsertLineEnd | ModalCommand::OpenBelow => coordinates.line_end(point)?,
            ModalCommand::OpenAbove => coordinates.line_start(point)?,
            ModalCommand::InsertLineFirst => {
                let start = coordinates.line_start(point)?;
                let end = coordinates.line_end(point)?;
                let body = buffer.slice(&buffer.span(start.0..end.0)?)?;
                let spaces = body.chars().take_while(|c| *c == ' ' || *c == '\t').count();
                Scalar(start.0 + spaces)
            }
        })
    }
}
