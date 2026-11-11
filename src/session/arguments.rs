mod history;
mod position;
mod text;
pub(super) use history::Replay;
pub(super) use text::Text;

use std::time::Instant;

use crate::command::{Command, KillCommand, LinearCommand, Motion, Scroll};
use crate::coordinates::Coordinates;
use crate::input::{Invocation, Outcome};
use crate::view::{Error, Size, View};

use super::{Action, Session};

#[derive(Clone, Copy)]
enum MovementKind {
    Motion(Motion),
    Row(Scroll),
    Line { backward: bool },
    Position,
}

/// Disposable planning progress. The selected view is published only after the
/// whole movement resolves; the existing deferred-input owner services C-g.
pub(super) struct Movement {
    invocation: Invocation,
    size: Size,
    candidate: View,
    remaining: usize,
    kind: MovementKind,
}

impl Session {
    pub(super) fn argument_feedback(&mut self) -> Action {
        if matches!(self.pending, Some(super::Pending::Command)) {
            self.pending = None;
            self.retire_command_origin();
        }
        let label = self.input.prefix_label().unwrap_or_default();
        self.focus_message(&label)
    }

    pub(super) fn execute_invocation(&mut self, invocation: Invocation) -> Action {
        if !self.input.invocation_current(invocation.origin) {
            if self.macros.active() {
                return self.stop_macro(
                    crate::macros::StopReason::OwnerChanged,
                    "Macro command argument origin changed",
                );
            }
            return self.message("Command argument origin changed");
        }
        if let Some(action) = self.execute_invocation_control(invocation) {
            return action;
        }
        if self.terminals.focused
            || self.capture_blocks_input()
            || self.tangling.is_some()
            || self.execution.is_some()
            || self.feeds.is_some()
        {
            return self.execute(invocation.command);
        }
        if let Some(action) = self.argument_tree(invocation) {
            return action;
        }
        self.execute_source_invocation(invocation)
    }

    fn execute_invocation_control(&mut self, invocation: Invocation) -> Option<Action> {
        if let Some(action) = self.macro_menu_command(invocation.command) {
            return Some(action);
        }
        if let Some(action) = self.macro_definition_guard(invocation.command) {
            return Some(action);
        }
        if let Some(action) = self.macro_command_guard(invocation.command) {
            return Some(action);
        }
        if let Command::PluginDefined(name) = invocation.command {
            return Some(self.plugin_counted(name, invocation.argument));
        }
        if let Command::Macro(control) = invocation.command {
            return Some(self.macro_command(control, invocation.argument));
        }
        if let Command::MacroNamed(name) = invocation.command {
            return Some(self.macro_named_call(name, invocation.argument));
        }
        if let Command::MacroBinding(id) = invocation.command {
            return Some(self.macro_bound_call(id, invocation.argument));
        }
        None
    }

    fn execute_source_invocation(&mut self, invocation: Invocation) -> Action {
        if self.source_layout_focus()
            && invocation.command == Command::GotoLine
            && invocation.argument.value() <= 0
        {
            return self.error("Goto line requires a positive argument");
        }
        let variant = match invocation.command {
            Command::ShellCommand => Some(Command::ShellCommandInsert),
            Command::ShellCommandOnRegion => Some(Command::ShellFilterRegion),
            _ => None,
        };
        if let Some(command) = variant {
            return self.execute(command);
        }
        if let Some(action) = self.execute_argument_entry(invocation) {
            return action;
        }
        if !self.argument_movement(invocation.command) {
            // Commands acquire numeric meaning only through their native policy.
            return self.execute(invocation.command);
        }
        if let Some(action) = self.layout_invocation(invocation) {
            return action;
        }
        self.publish_argument_movement(invocation)
    }

    fn execute_argument_entry(&mut self, invocation: Invocation) -> Option<Action> {
        if self.source_layout_focus() {
            if matches!(invocation.command, Command::Newline | Command::InsertTab) {
                return Some(self.execute_argument_insertion(invocation));
            }
            if let Command::History(direction) = invocation.command {
                return Some(self.execute_argument_history(invocation, direction));
            }
            if matches!(
                invocation.command,
                Command::Linear(LinearCommand::Yank | LinearCommand::Pop)
            ) {
                return Some(self.argument_yank(invocation));
            }
        }
        None
    }

    pub(super) fn argument_movement(&self, command: Command) -> bool {
        self.source_layout_focus()
            && matches!(
                command,
                Command::Motion(
                    Motion::ForwardChar
                        | Motion::BackwardChar
                        | Motion::ForwardWord
                        | Motion::BackwardWord
                        | Motion::LineStart
                        | Motion::LineEnd
                        | Motion::BufferStart
                        | Motion::BufferEnd
                ) | Command::Scroll(_)
                    | Command::GotoLine
                    | Command::DeleteChar
                    | Command::BackwardDeleteChar
                    | Command::Kill(_)
            )
    }

    pub(super) fn prepare_argument_movement(
        &mut self,
        invocation: Invocation,
    ) -> Result<(), Error> {
        self.admit_argument_movement(invocation)?;
        let size = self.pane_size();
        let mut movement = self.argument_movement_work(invocation, size);
        let region = self.argument_deletion_region(invocation)?;
        let result = if matches!(movement.kind, MovementKind::Position) {
            self.prepare_argument_position(&mut movement)
        } else if region.is_some()
            || invocation.argument.value() == 0 && zero_inert(invocation.command)
        {
            movement.remaining = 0;
            Ok(())
        } else if invocation.argument.value() == 0
            && invocation.command == Command::Kill(KillCommand::Line)
        {
            self.prepare_zero_line_argument(&mut movement)?;
            Ok(())
        } else {
            self.advance_argument_movement(&mut movement)
        };
        let result = result.and_then(|()| self.finish_argument_row(&mut movement));
        self.layout.movement = Some(movement);
        result
    }

    fn admit_argument_movement(&self, invocation: Invocation) -> Result<(), Error> {
        if !self.input.invocation_current(invocation.origin)
            || !self.argument_movement(invocation.command)
        {
            return Err(crate::buffer::edit::Error::ChangedPositions.into());
        }
        Ok(())
    }

    fn argument_movement_work(&mut self, invocation: Invocation, size: Size) -> Movement {
        self.layout
            .movement
            .take()
            .filter(|work| work.invocation == invocation && work.size == size)
            .unwrap_or_else(|| Movement {
                invocation,
                size,
                candidate: self.view().clone(),
                remaining: argument_distance(invocation),
                kind: movement_kind(invocation.command, argument_reverse(invocation)),
            })
    }

    fn argument_deletion_region(
        &self,
        invocation: Invocation,
    ) -> Result<Option<crate::buffer::Span>, Error> {
        let deletion = matches!(
            invocation.command,
            Command::DeleteChar | Command::BackwardDeleteChar
        );
        Ok(deletion
            .then(|| self.buffer()?.region(self.view().point))
            .transpose()?
            .flatten()
            .filter(|span| !span.range.is_empty()))
    }

    fn prepare_zero_line_argument(&self, movement: &mut Movement) -> Result<(), Error> {
        movement.candidate.point.position =
            Coordinates::new(self.buffer()?)?.line_start(self.view().point.position)?;
        movement.remaining = 0;
        Ok(())
    }

    fn advance_argument_movement(&self, work: &mut Movement) -> Result<(), Error> {
        let buffer = self.buffer()?;
        work.candidate.check_source(buffer)?;
        let coordinates = Coordinates::new(buffer)?;
        let started = Instant::now();
        for _ in 0..256 {
            if work.remaining == 0 {
                return Ok(());
            }
            let point = work.candidate.point.position;
            work.advance(buffer, &coordinates)?;
            work.remaining -= 1;
            if work.candidate.point.position == point {
                work.remaining = 0;
            }
            if started.elapsed() >= super::events::INPUT_BATCH_TIME {
                break;
            }
        }
        if work.remaining == 0 {
            Ok(())
        } else {
            Err(crate::coordinates::Error::Pending.into())
        }
    }

    pub(super) fn publish_argument_movement(&mut self, invocation: Invocation) -> Action {
        let Some(work) = self.layout.movement.take() else {
            return self.message("Command argument movement retired");
        };
        if work.invocation != invocation
            || !self.input.invocation_current(invocation.origin)
            || work.remaining != 0
        {
            return self.message("Command argument origin changed");
        }
        if invocation.argument.value() == 0 && zero_inert(invocation.command) {
            return self.message("");
        }
        if matches!(
            invocation.command,
            Command::DeleteChar | Command::BackwardDeleteChar | Command::Kill(_)
        ) {
            return self.publish_argument_range(&work);
        }
        self.command_boundary(invocation.command);
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        let result = self.place_view(work.candidate);
        self.sync_input_context();
        match result {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn publish_argument_range(&mut self, work: &Movement) -> Action {
        let command = work.invocation.command;
        let point = self.view().point.position.0;
        let target = work.candidate.point.position.0;
        let result = (|| {
            let range = self.argument_edit_range(command, point.min(target)..point.max(target))?;
            if range.is_empty() {
                return Ok(false);
            }
            self.command_boundary(command);
            self.stop_typing()?;
            self.finish_repeat_entry();
            if matches!(command, Command::Kill(_)) {
                let source = self.buffer()?.span(range)?;
                self.kill_source(source, target < point)
            } else {
                self.replace(range, "")
            }
        })();
        self.sync_input_context();
        self.finish(result)
    }

    fn argument_edit_range(
        &self,
        command: Command,
        range: std::ops::Range<usize>,
    ) -> Result<std::ops::Range<usize>, crate::buffer::edit::Error> {
        if matches!(command, Command::DeleteChar | Command::BackwardDeleteChar)
            && let Some(region) = self.buffer()?.region(self.view().point)?
            && !region.range.is_empty()
        {
            return Ok(region.range);
        }
        Ok(range)
    }

    pub(super) fn execute_prefix_control(&mut self, command: Command) -> Action {
        let outcome = self.input.prefix_control(command, None);
        match outcome {
            Outcome::ArgumentPrefix(_) => self.argument_feedback(),
            Outcome::Message(message) => self.message(message),
            _ => unreachable!("prefix helper emits input feedback"),
        }
    }
}

impl Movement {
    fn advance(
        &mut self,
        buffer: &crate::buffer::Buffer,
        coordinates: &Coordinates<'_>,
    ) -> Result<(), Error> {
        match self.kind {
            MovementKind::Position => unreachable!("absolute positions prepare separately"),
            MovementKind::Motion(motion) => self.advance_motion(coordinates, motion),
            MovementKind::Line { backward } => self.advance_line(coordinates, backward),
            MovementKind::Row(scroll) => self.advance_row(buffer, scroll),
        }
    }

    fn advance_motion(
        &mut self,
        coordinates: &Coordinates<'_>,
        motion: Motion,
    ) -> Result<(), Error> {
        let point = self.candidate.point.position;
        self.candidate.point.position = match motion {
            Motion::ForwardChar => coordinates.next_observed(point)?,
            Motion::BackwardChar => coordinates.previous_observed(point)?,
            _ => motion.target(coordinates, point)?,
        };
        self.candidate.reset_navigation();
        Ok(())
    }

    fn advance_line(&mut self, coordinates: &Coordinates<'_>, backward: bool) -> Result<(), Error> {
        let point = self.candidate.point.position;
        self.candidate.point.position = if backward {
            let start = coordinates.line_start(point)?;
            if start.0 == 0 {
                start
            } else {
                coordinates.line_start(coordinates.previous_observed(start)?)?
            }
        } else {
            coordinates.next_observed(coordinates.line_end(point)?)?
        };
        Ok(())
    }

    fn advance_row(&mut self, buffer: &crate::buffer::Buffer, scroll: Scroll) -> Result<(), Error> {
        let width = self.candidate.text_width(buffer, self.size)?;
        if buffer.soft_wrap() && width > 0 {
            self.candidate.visual_scroll(buffer, width, scroll, 1)?;
        } else {
            self.candidate.scroll(buffer, scroll, 1)?;
        }
        Ok(())
    }
}

fn movement_kind(command: Command, reverse: bool) -> MovementKind {
    match command {
        Command::GotoLine | Command::Scroll(Scroll::Recenter) => MovementKind::Position,
        Command::Motion(motion) => motion_kind(motion, reverse),
        Command::DeleteChar | Command::BackwardDeleteChar | Command::Kill(_) => {
            deletion_kind(command, reverse)
        }
        Command::Scroll(scroll) => MovementKind::Row(if reverse {
            reverse_scroll(scroll)
        } else {
            scroll
        }),
        _ => unreachable!("counted movement policy checked before preparation"),
    }
}

fn motion_kind(motion: Motion, reverse: bool) -> MovementKind {
    match motion {
        Motion::BufferStart | Motion::BufferEnd => MovementKind::Position,
        Motion::LineStart | Motion::LineEnd => MovementKind::Row(if reverse {
            Scroll::PreviousLine
        } else {
            Scroll::NextLine
        }),
        _ => MovementKind::Motion(if reverse {
            reverse_motion(motion)
        } else {
            motion
        }),
    }
}

fn deletion_kind(command: Command, reverse: bool) -> MovementKind {
    let motion = match command {
        Command::DeleteChar => Motion::ForwardChar,
        Command::BackwardDeleteChar => Motion::BackwardChar,
        Command::Kill(KillCommand::Word) => Motion::ForwardWord,
        Command::Kill(KillCommand::BackwardWord) => Motion::BackwardWord,
        Command::Kill(KillCommand::Line) => return MovementKind::Line { backward: reverse },
        _ => unreachable!("only deletions have a counted range"),
    };
    motion_kind(motion, reverse)
}

fn reverse_motion(motion: Motion) -> Motion {
    match motion {
        Motion::ForwardChar => Motion::BackwardChar,
        Motion::BackwardChar => Motion::ForwardChar,
        Motion::ForwardWord => Motion::BackwardWord,
        Motion::BackwardWord => Motion::ForwardWord,
        _ => unreachable!("only distance motions admit signed arguments"),
    }
}

fn reverse_scroll(scroll: Scroll) -> Scroll {
    match scroll {
        Scroll::NextLine => Scroll::PreviousLine,
        Scroll::PreviousLine => Scroll::NextLine,
        Scroll::PageForward => Scroll::PageBackward,
        Scroll::PageBackward => Scroll::PageForward,
        Scroll::Recenter => unreachable!(),
    }
}

fn argument_distance(invocation: Invocation) -> usize {
    if matches!(
        invocation.command,
        Command::Motion(Motion::LineStart | Motion::LineEnd)
    ) {
        (invocation.argument.value() - 1).unsigned_abs() as usize
    } else {
        invocation.argument.value().unsigned_abs() as usize
    }
}

fn argument_reverse(invocation: Invocation) -> bool {
    if matches!(
        invocation.command,
        Command::Motion(Motion::LineStart | Motion::LineEnd)
    ) {
        invocation.argument.value() < 1
    } else {
        invocation.argument.value() < 0
    }
}

fn zero_inert(command: Command) -> bool {
    !matches!(
        command,
        Command::Kill(KillCommand::Line)
            | Command::Motion(
                Motion::LineStart | Motion::LineEnd | Motion::BufferStart | Motion::BufferEnd
            )
            | Command::GotoLine
            | Command::Scroll(Scroll::Recenter)
    )
}
