use crate::command::{Command, Motion, Scroll, TreeCommand};
use crate::coordinates::{Coordinates, LineByte, Scalar};
use crate::input::Invocation;
use crate::session::{Action, Session};
use crate::view::Error;

use super::Movement;

impl Session {
    pub(super) fn prepare_argument_position(&self, work: &mut Movement) -> Result<(), Error> {
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        match work.invocation.command {
            Command::Motion(Motion::BufferStart | Motion::BufferEnd) => {
                work.prepare_fraction(buffer, &coordinates)?;
            }
            Command::GotoLine => work.prepare_line(buffer, &coordinates)?,
            Command::Scroll(Scroll::Recenter) => self.prepare_argument_recenter(work, buffer)?,
            _ => unreachable!("only absolute positioning uses this owner"),
        }
        work.remaining = 0;
        Ok(())
    }

    fn prepare_argument_recenter(
        &self,
        work: &mut Movement,
        buffer: &crate::buffer::Buffer,
    ) -> Result<(), Error> {
        let value = work.invocation.argument.value();
        let rows = usize::from(self.pane_size().text_rows());
        let offset = if value >= 0 {
            usize::try_from(value).expect("nonnegative recenter argument")
        } else {
            rows.saturating_sub(value.unsigned_abs() as usize)
        };
        work.candidate
            .recenter_at(buffer, self.pane_size(), offset)?;
        Ok(())
    }

    pub(super) fn finish_argument_row(&self, work: &mut Movement) -> Result<(), Error> {
        let Command::Motion(motion @ (Motion::LineStart | Motion::LineEnd)) =
            work.invocation.command
        else {
            return Ok(());
        };
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        let width = work.candidate.text_width(buffer, work.size)?;
        work.candidate.point.position = if buffer.soft_wrap() && width > 0 {
            work.candidate
                .visual_boundary(buffer, width, motion == Motion::LineEnd)?
        } else {
            motion.target(&coordinates, work.candidate.point.position)?
        };
        work.candidate.reset_navigation();
        Ok(())
    }

    pub(super) fn argument_tree(&mut self, invocation: Invocation) -> Option<Action> {
        if !self.tree.focused() {
            return None;
        }
        let command = match invocation.command {
            Command::Tree(command) => command,
            command => TreeCommand::alias(command)?,
        };
        if !matches!(command, TreeCommand::Next | TreeCommand::Previous) {
            return None;
        }
        let count = invocation.argument.value();
        if count != 0 {
            let forward = (command == TreeCommand::Next) != (count < 0);
            self.command_boundary(Command::Tree(command));
            if let Err(error) = self.stop_typing() {
                return Some(self.error(&error.to_string()));
            }
            self.tree
                .move_selection_by(forward, count.unsigned_abs() as usize);
            self.sync_input_context();
        }
        Some(self.message("File tree updated"))
    }
}

impl Movement {
    fn prepare_fraction(
        &mut self,
        buffer: &crate::buffer::Buffer,
        coordinates: &Coordinates<'_>,
    ) -> Result<(), Error> {
        let value = self.invocation.argument.value();
        let len = buffer.len_chars() as i128;
        let distance = len * i128::from(value) / 10;
        let target = if self.invocation.command == Command::Motion(Motion::BufferStart) {
            distance
        } else {
            len - distance
        }
        .clamp(0, len);
        let point = Scalar(usize::try_from(target).expect("clamped buffer position"));
        self.candidate.point.position = if coordinates.is_boundary(point)? {
            point
        } else {
            coordinates.previous(point)?
        };
        self.candidate.reset_navigation();
        Ok(())
    }

    fn prepare_line(
        &mut self,
        buffer: &crate::buffer::Buffer,
        coordinates: &Coordinates<'_>,
    ) -> Result<(), Error> {
        let value = self.invocation.argument.value();
        let row = usize::try_from(value - 1).expect("positive line argument");
        self.candidate.point.position = if row < buffer.len_lines() {
            coordinates.from_line(LineByte { row, column: 0 })?
        } else {
            coordinates.end()
        };
        self.candidate.reset_navigation();
        Ok(())
    }
}
