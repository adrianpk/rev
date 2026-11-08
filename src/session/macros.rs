mod capture;
mod controls;
mod counters;
mod literal;
mod repeat;
mod runner;

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::Instant;

use crossterm::event::Event;

use super::{Action, Session};
use crate::command::{Command, MacroCommand, PrefixArgument};
use crate::input::{Focus, Input};
use crate::macros::{Entry, Recorder, Runner, Status, Stop, StopReason};

impl Session {
    pub fn macro_status(&self) -> Status {
        self.macros.status()
    }

    pub fn macro_stop(&self) -> Option<Stop> {
        self.macros.stop
    }

    /// Observe native layout or fresh-consent ownership without advancing it.
    /// Consent has no automatic macro wake; the host waits for ordinary input.
    pub fn macro_wait(&self) -> Option<crate::macros::Wait> {
        self.macros.runner.as_ref().and_then(|run| {
            if run.consent.is_some() {
                Some(crate::macros::Wait::Consent)
            } else if run.waiting {
                Some(crate::macros::Wait::Layout)
            } else {
                None
            }
        })
    }

    fn macro_entry(&self) -> Entry {
        Entry {
            profile: self.input.interaction_profile(),
            mode: self.input.editing_state(),
        }
    }

    fn macro_source(&self) -> bool {
        self.source_layout_focus()
            && self.input.focus() == Focus::Editing
            && !self.terminals.focused
            && self.startup_screen.is_none()
            && !self.git_visible()
            && !self.bookmark_list_active()
            && self.pending.is_none()
            && self.completion.is_none()
    }

    pub(super) fn stop_macro(&mut self, reason: StopReason, message: &str) -> Action {
        if self.macros.active() {
            self.restore_search();
            self.pending = None;
        }
        self.macros.fail(reason);
        self.buffers.automatic_group(None);
        self.input.cancel();
        let detail = self.macros.stop.map_or(String::new(), |stop| {
            if stop.iteration == 0 {
                String::new()
            } else {
                format!(" (iteration {}, step {})", stop.iteration, stop.step)
            }
        });
        self.message(&format!("{message}{detail}"))
    }

    /// Physical input cannot interleave with an automatic step. Resize and
    /// cancellation retain their native owners; the run itself never blocks IO.
    pub(super) fn macro_input_guard(&mut self, event: &Event) -> Option<Action> {
        if self.macros.recorder.is_some() && matches!(event, Event::Mouse(_)) {
            return Some(self.stop_macro(
                StopReason::Unsupported,
                "Keyboard macro recording stopped before pointer input",
            ));
        }
        if self.macros.expanded || self.macros.runner.is_none() {
            return None;
        }
        if Input::interrupt(event) {
            self.stop_macro(StopReason::Canceled, "Keyboard macro canceled");
            return None;
        }
        if self
            .macros
            .runner
            .as_ref()
            .is_some_and(|run| run.consent.is_some())
        {
            let current = (self.windows.selected_id(), self.view().point);
            if self
                .macros
                .runner
                .as_ref()
                .is_some_and(|run| run.expected != current)
            {
                return Some(
                    self.stop_macro(StopReason::OwnerChanged, "Macro consent source changed"),
                );
            }
            return None;
        }
        if matches!(event, Event::Paste(_) | Event::Mouse(_))
            || matches!(event, Event::Key(key) if key.kind != crossterm::event::KeyEventKind::Release)
        {
            return Some(self.message("Keyboard macro is running; C-g cancels"));
        }
        None
    }

    pub(super) fn macro_command_guard(&mut self, command: Command) -> Option<Action> {
        if command == Command::KeyboardQuit {
            self.macros.repeat = None;
        }
        if !self.macros.active() {
            return None;
        }
        if command == Command::KeyboardQuit {
            self.stop_macro(StopReason::Canceled, "Keyboard macro canceled");
            return None;
        }
        if matches!(command, Command::Macro(_)) && !self.macros.expanded {
            return None;
        }
        if self.macros.expanded
            && matches!(
                command,
                Command::Macro(
                    MacroCommand::InsertCounter
                        | MacroCommand::SetCounter
                        | MacroCommand::AddCounter
                        | MacroCommand::SetFormat
                        | MacroCommand::Redisplay
                )
            )
        {
            return None;
        }
        if self.macros.expanded && matches!(command, Command::Completion(_)) {
            return Some(self.stop_macro(
                StopReason::Unsupported,
                "Macro completion requires a captured literal edit",
            ));
        }
        if (!self.macros.expanded && !self.macros.dispatching) || !supported(command) {
            return Some(self.stop_macro(
                StopReason::Unsupported,
                "Keyboard macro stopped before unsupported command",
            ));
        }
        None
    }

    pub(super) fn macro_command(
        &mut self,
        command: MacroCommand,
        argument: PrefixArgument,
    ) -> Action {
        self.finish_repeat_entry();
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        if command == MacroCommand::StartOrCounter && self.macros.recorder.is_some() {
            return self.macro_counter_command(MacroCommand::InsertCounter, argument);
        }
        if matches!(
            command,
            MacroCommand::InsertCounter
                | MacroCommand::SetCounter
                | MacroCommand::AddCounter
                | MacroCommand::SetFormat
        ) {
            return self.macro_counter_command(command, argument);
        }
        if command == MacroCommand::Redisplay {
            if let Some(recorder) = &mut self.macros.recorder
                && let Err(reason) = recorder
                    .stage(crate::macros::Step::Redisplay)
                    .and_then(|()| recorder.accept())
            {
                return self.stop_macro(reason, "Keyboard macro recording limit exceeded");
            }
            return self.message("");
        }
        if self.macros.expanded || self.macros.runner.is_some() {
            return self.stop_macro(
                StopReason::Unsupported,
                "Nested macro controls are unavailable",
            );
        }
        if matches!(command, MacroCommand::Start | MacroCommand::StartOrCounter) {
            return self.macro_start_recording(argument);
        }
        if matches!(
            command,
            MacroCommand::Next
                | MacroCommand::Previous
                | MacroCommand::Delete
                | MacroCommand::Swap
                | MacroCommand::CallSecond
        ) {
            return self.macro_ring_command(command, argument);
        }
        let recorded = self.macros.recorder.is_some();
        if let Some(recorder) = self.macros.recorder.take() {
            if recorder.steps.len() == recorder.original_len {
                self.macros.stop = Some(Stop {
                    reason: StopReason::Empty,
                    iteration: 0,
                    step: 0,
                });
                return self.message("Empty recording discarded; previous macro retained");
            }
            let limit = self.buffers.snapshot().settings().macros.ring_limit;
            if let Err(reason) = self.macros.publish(recorder, limit) {
                return self.stop_macro(reason, "Keyboard macro definition revision limit reached");
            }
        } else if command == MacroCommand::End {
            return self.message("No keyboard macro is being recorded");
        }
        let count = if recorded {
            match command {
                MacroCommand::EndOrCall => 0,
                MacroCommand::End if argument.value() > 0 => {
                    if argument.supplied() {
                        argument.value() - 1
                    } else {
                        0
                    }
                }
                _ => argument.value(),
            }
        } else {
            argument.value()
        };
        self.macros.repeat = match command {
            MacroCommand::EndAndCall => Some(crate::macros::Repeat::EndCall),
            MacroCommand::CallRepeat => Some(crate::macros::Repeat::Ring),
            _ => None,
        };
        if count == 0
            && recorded
            && (command == MacroCommand::EndOrCall || !argument.supplied() || argument.value() > 0)
        {
            return self.message("Keyboard macro recorded");
        }
        self.start_macro(count)
    }

    fn start_macro(&mut self, count: i32) -> Action {
        let Some(definition) = self.macros.ring.front().cloned() else {
            return self.message("No keyboard macro is available");
        };
        if definition.steps.is_empty() {
            return self.stop_macro(StopReason::Empty, "Empty keyboard macro cannot run");
        }
        if !self.macro_source() || definition.entry != self.macro_entry() {
            self.macros.stop = Some(Stop {
                reason: StopReason::EntryState,
                iteration: 1,
                step: 0,
            });
            return self.message("Keyboard macro entry profile or source state differs");
        }
        let budget = Arc::new(AtomicUsize::new(0));
        self.macros.stop = None;
        let mut counter = definition.counter.clone();
        counter.iteration_start = counter.current;
        counter.format = definition.initial_format.clone();
        self.macros.runner = Some(Runner {
            counter,
            append: false,
            definition,
            iterations: (count > 0).then_some(count as usize),
            iteration: 0,
            step: 0,
            dispatched: 0,
            started: Instant::now(),
            expected: (self.windows.selected_id(), self.view().point),
            waiting: false,
            group: crate::buffer::edit::AutomaticGroup::new(budget.clone()),
            budget,
            arguments: Default::default(),
            trigger: Default::default(),
            consent: None,
        });
        self.message("Keyboard macro playing")
    }
}

#[cfg(test)]
mod tests;

fn supported(command: Command) -> bool {
    if matches!(
        command,
        Command::Completion(
            crate::command::CompletionCommand::Mode | crate::command::CompletionCommand::AutoMode
        )
    ) {
        return false;
    }
    matches!(
        command,
        Command::Motion(_)
            | Command::Navigate(_)
            | Command::Edit(_)
            | Command::Modal(_)
            | Command::Scroll(_)
            | Command::History(_)
            | Command::Region(_)
            | Command::Rectangle(_)
            | Command::Kill(_)
            | Command::Linear(_)
            | Command::Window(_)
            | Command::Split(_)
            | Command::Newline
            | Command::InsertTab
            | Command::BackwardDeleteChar
            | Command::DeleteChar
            | Command::PrefixArgument
            | Command::UniversalArgument
            | Command::DigitArgument
            | Command::NegativeArgument
            | Command::ExecuteExtended
            | Command::SearchForward
            | Command::SearchBackward
            | Command::QueryReplace
            | Command::FindFile
            | Command::GotoLine
            | Command::SaveBuffer
            | Command::WriteFile
            | Command::Buffer(_)
            | Command::GenerateNewBuffer
            | Command::Completion(_)
    )
}
