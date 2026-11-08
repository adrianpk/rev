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
        if let Some(action) = self.macro_command_guard(invocation.command) {
            return action;
        }
        if let Command::Macro(control) = invocation.command {
            return self.macro_command(control, invocation.argument);
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
        if self.source_layout_focus() {
            if matches!(invocation.command, Command::Newline | Command::InsertTab) {
                return self.execute_argument_insertion(invocation);
            }
            if let Command::History(direction) = invocation.command {
                return self.execute_argument_history(invocation, direction);
            }
            if matches!(
                invocation.command,
                Command::Linear(LinearCommand::Yank | LinearCommand::Pop)
            ) {
                return self.argument_yank(invocation);
            }
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
        if !self.input.invocation_current(invocation.origin)
            || !self.argument_movement(invocation.command)
        {
            return Err(crate::buffer::edit::Error::ChangedPositions.into());
        }
        let size = self.pane_size();
        let mut movement = self
            .layout
            .movement
            .take()
            .filter(|work| work.invocation == invocation && work.size == size)
            .unwrap_or_else(|| Movement {
                invocation,
                size,
                candidate: self.view().clone(),
                remaining: argument_distance(invocation),
                kind: movement_kind(invocation.command, argument_reverse(invocation)),
            });
        let deletion = matches!(
            invocation.command,
            Command::DeleteChar | Command::BackwardDeleteChar
        );
        let region = deletion
            .then(|| self.buffer()?.region(self.view().point))
            .transpose()?
            .flatten()
            .filter(|span| !span.range.is_empty());
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
            movement.candidate.point.position =
                Coordinates::new(self.buffer()?)?.line_start(self.view().point.position)?;
            movement.remaining = 0;
            Ok(())
        } else {
            self.advance_argument_movement(&mut movement)
        };
        let result = result.and_then(|()| self.finish_argument_row(&mut movement));
        self.layout.movement = Some(movement);
        result
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
            match work.kind {
                MovementKind::Position => unreachable!("absolute positions prepare separately"),
                MovementKind::Motion(motion) => {
                    work.candidate.point.position = match motion {
                        Motion::ForwardChar => coordinates.next_observed(point)?,
                        Motion::BackwardChar => coordinates.previous_observed(point)?,
                        _ => motion.target(&coordinates, point)?,
                    };
                    work.candidate.reset_navigation();
                }
                MovementKind::Line { backward } => {
                    work.candidate.point.position = if backward {
                        let start = coordinates.line_start(point)?;
                        if start.0 == 0 {
                            start
                        } else {
                            coordinates.line_start(coordinates.previous_observed(start)?)?
                        }
                    } else {
                        coordinates.next_observed(coordinates.line_end(point)?)?
                    };
                }
                MovementKind::Row(scroll) => {
                    let width = work.candidate.text_width(buffer, work.size)?;
                    if buffer.soft_wrap() && width > 0 {
                        work.candidate.visual_scroll(buffer, width, scroll, 1)?;
                    } else {
                        work.candidate.scroll(buffer, scroll, 1)?;
                    }
                }
            }
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
            return self.publish_argument_range(work);
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

    fn publish_argument_range(&mut self, work: Movement) -> Action {
        let command = work.invocation.command;
        let point = self.view().point.position.0;
        let target = work.candidate.point.position.0;
        let result = (|| {
            let mut range = point.min(target)..point.max(target);
            if matches!(command, Command::DeleteChar | Command::BackwardDeleteChar)
                && let Some(region) = self.buffer()?.region(self.view().point)?
                && !region.range.is_empty()
            {
                range = region.range;
            }
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

    pub(super) fn execute_prefix_control(&mut self, command: Command) -> Action {
        let outcome = self.input.prefix_control(command, None);
        match outcome {
            Outcome::ArgumentPrefix(_) => self.argument_feedback(),
            Outcome::Message(message) => self.message(message),
            _ => unreachable!("prefix helper emits input feedback"),
        }
    }
}

fn movement_kind(command: Command, reverse: bool) -> MovementKind {
    match command {
        Command::Motion(Motion::BufferStart | Motion::BufferEnd)
        | Command::GotoLine
        | Command::Scroll(Scroll::Recenter) => MovementKind::Position,
        Command::Motion(Motion::LineStart | Motion::LineEnd) => MovementKind::Row(if reverse {
            Scroll::PreviousLine
        } else {
            Scroll::NextLine
        }),
        Command::Motion(motion) => MovementKind::Motion(if reverse {
            match motion {
                Motion::ForwardChar => Motion::BackwardChar,
                Motion::BackwardChar => Motion::ForwardChar,
                Motion::ForwardWord => Motion::BackwardWord,
                Motion::BackwardWord => Motion::ForwardWord,
                _ => unreachable!("only distance motions admit signed arguments"),
            }
        } else {
            motion
        }),
        Command::DeleteChar | Command::BackwardDeleteChar => movement_kind(
            Command::Motion(if command == Command::DeleteChar {
                Motion::ForwardChar
            } else {
                Motion::BackwardChar
            }),
            reverse,
        ),
        Command::Kill(KillCommand::Word | KillCommand::BackwardWord) => movement_kind(
            Command::Motion(if command == Command::Kill(KillCommand::Word) {
                Motion::ForwardWord
            } else {
                Motion::BackwardWord
            }),
            reverse,
        ),
        Command::Kill(KillCommand::Line) => MovementKind::Line { backward: reverse },
        Command::Scroll(scroll) => MovementKind::Row(if reverse {
            match scroll {
                Scroll::NextLine => Scroll::PreviousLine,
                Scroll::PreviousLine => Scroll::NextLine,
                Scroll::PageForward => Scroll::PageBackward,
                Scroll::PageBackward => Scroll::PageForward,
                Scroll::Recenter => unreachable!(),
            }
        } else {
            scroll
        }),
        _ => unreachable!("counted movement policy checked before preparation"),
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
