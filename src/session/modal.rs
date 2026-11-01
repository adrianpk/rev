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
        if self.interaction_profile() != InteractionProfile::Modal {
            return self.message("This command requires the modal startup profile");
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
        if command == ModalCommand::Once {
            return self.begin_one_normal();
        }
        let result = if let Some(shape) = command.visual() {
            self.enter_visual(shape)
        } else {
            self.enter_modal(command)
        };
        match result {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn enter_modal(&mut self, command: ModalCommand) -> Result<(), Error> {
        let target = self.modal_target(command)?;
        self.leave_visual();
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
        self.input.set_editing_state(entry_state(command));
        Ok(())
    }

    /// Insert exit retains the native point. Do not decrement a byte or
    /// synthesize a separate Normal cursor.
    pub(super) fn modal_target(&self, command: ModalCommand) -> Result<Scalar, Error> {
        let point = self.view().point.position;
        let buffer = self.buffer()?;
        self.view().check_point(buffer)?;
        let coordinates = Coordinates::new(buffer)?;
        entry_target(&coordinates, point, command).map_err(Error::from)
    }

    pub(super) fn modal_dispatch(&mut self, command: crate::command::Command) -> Option<Action> {
        let crate::command::Command::Modal(command) = command else {
            return None;
        };
        Some(self.modal_command(command))
    }

    pub(super) fn modal_frame(
        &self,
        frame: &mut crate::display::Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), Error> {
        let Some(state) = self.editing_state() else {
            return Ok(());
        };
        let Some((_, panel)) = composition.and_then(|c| {
            c.panels()
                .iter()
                .find(|(id, _)| *id == self.windows.selected_id())
        }) else {
            return Ok(());
        };
        let buffer = self.buffer()?;
        frame.status(buffer, self.view().cell(buffer)?, *panel, Some(state))
    }
}

fn entry_target(
    coordinates: &Coordinates<'_>,
    point: Scalar,
    command: ModalCommand,
) -> Result<Scalar, crate::coordinates::Error> {
    match command {
        ModalCommand::Normal | ModalCommand::Insert => Ok(point),
        ModalCommand::InsertAfter => {
            let next = coordinates.next_observed(point)?;
            coordinates.line_end(point).map(|end| next.min(end))
        }
        ModalCommand::InsertLineEnd | ModalCommand::OpenBelow => coordinates.line_end(point),
        ModalCommand::OpenAbove => coordinates.line_start(point),
        ModalCommand::InsertLineFirst => first_text(coordinates, point),
        _ => Ok(point),
    }
}

fn first_text(
    coordinates: &Coordinates<'_>,
    point: Scalar,
) -> Result<Scalar, crate::coordinates::Error> {
    let row = coordinates.to_line(point)?.row;
    for glyph in coordinates.glyphs(row)? {
        let glyph = glyph?;
        if !glyph.text.chars().all(|c| c == ' ' || c == '\t') {
            return Ok(glyph.source.start);
        }
    }
    coordinates.line_end(point)
}

fn entry_state(command: ModalCommand) -> EditingState {
    match command {
        ModalCommand::Normal => EditingState::Normal,
        ModalCommand::Replace => EditingState::Replace,
        _ => EditingState::Insert,
    }
}
