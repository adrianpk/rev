mod runner;

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::Instant;

use crossterm::event::Event;

use super::{Action, Session};
use crate::command::{Command, MacroCommand, PrefixArgument};
use crate::input::{Focus, Input, Outcome};
use crate::macros::{Definition, Entry, Recorder, Runner, Status, Step, Stop, StopReason};

impl Session {
    pub fn macro_status(&self) -> Status {
        self.macros.status()
    }

    pub fn macro_stop(&self) -> Option<Stop> {
        self.macros.stop
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

    fn stop_macro(&mut self, reason: StopReason, message: &str) -> Action {
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
        if matches!(event, Event::Paste(_) | Event::Mouse(_))
            || matches!(event, Event::Key(key) if key.kind != crossterm::event::KeyEventKind::Release)
        {
            return Some(self.message("Keyboard macro is running; C-g cancels"));
        }
        None
    }

    pub(super) fn macro_command_guard(&mut self, command: Command) -> Option<Action> {
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
        if (!self.macros.expanded && !self.macros.dispatching) || !supported(command) {
            return Some(self.stop_macro(
                StopReason::Unsupported,
                "Keyboard macro stopped before unsupported command",
            ));
        }
        None
    }

    pub(super) fn macro_capture_outcome(&mut self, outcome: &Outcome) -> Option<Action> {
        if self.macros.expanded || self.macros.recorder.is_none() {
            return None;
        }
        let command = match outcome {
            Outcome::Command(command) => Some(*command),
            Outcome::Invocation(invocation) => Some(invocation.command),
            _ => None,
        };
        if command.is_some_and(|c| matches!(c, Command::Macro(_))) {
            return None;
        }
        if command == Some(Command::KeyboardQuit) {
            self.macros.fail(StopReason::Canceled);
            return None;
        }
        if matches!(outcome, Outcome::Ignored) {
            return None;
        }
        if matches!(outcome, Outcome::Message(_) | Outcome::Pointer(_)) {
            return Some(self.stop_macro(
                StopReason::Unsupported,
                "Keyboard macro recording stopped before unsupported input",
            ));
        }
        if matches!(outcome, Outcome::Paste(text) if text.is_empty()) {
            return None;
        }
        if matches!(outcome, Outcome::Paste(text) if text.contains('\0')) {
            return Some(self.stop_macro(
                StopReason::Unsupported,
                "Keyboard macro recording stopped before rejected paste",
            ));
        }
        let text_bytes = match outcome {
            Outcome::Paste(text) => text.len(),
            Outcome::TextInvocation(text) => text.text.len(),
            _ => 0,
        };
        if text_bytes > crate::macros::DEFINITION_BYTES {
            return Some(self.stop_macro(
                StopReason::CaptureLimit,
                "Keyboard macro recording limit exceeded",
            ));
        }
        let step = self
            .input
            .logical_key
            .map(Step::Key)
            .or_else(|| match outcome {
                Outcome::Paste(text) => Some(Step::Text(text.clone())),
                Outcome::TextInvocation(text) => Some(Step::Text(text.text.clone())),
                _ => None,
            });
        if let Some(step) = step {
            let recorder = self.macros.recorder.as_mut().expect("capture owner");
            if let Err(reason) = recorder.stage(step) {
                return Some(self.stop_macro(reason, "Keyboard macro recording limit exceeded"));
            }
            if !matches!(
                outcome,
                Outcome::Prefix(_) | Outcome::ArgumentPrefix(_) | Outcome::Changed
            ) && let Err(reason) = recorder.accept()
            {
                return Some(self.stop_macro(reason, "Keyboard macro recording limit exceeded"));
            }
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
        if self.macros.expanded || self.macros.runner.is_some() {
            return self.stop_macro(
                StopReason::Unsupported,
                "Nested macro controls are unavailable",
            );
        }
        if matches!(command, MacroCommand::Start | MacroCommand::StartOrCounter) {
            if self.macros.recorder.is_some() || argument.supplied() {
                return self.stop_macro(
                    StopReason::Unsupported,
                    "Macro append and counters are not available in this delivery increment",
                );
            }
            if !self.macro_source() {
                return self.message("Keyboard macro requires ordinary source editing");
            }
            self.macros.stop = None;
            self.macros.recorder = Some(Recorder::new(self.macro_entry()));
            return self.message("Def: recording keyboard macro");
        }
        let recorded = self.macros.recorder.is_some();
        if let Some(recorder) = self.macros.recorder.take() {
            if recorder.steps.is_empty() {
                self.macros.stop = Some(Stop {
                    reason: StopReason::Empty,
                    iteration: 0,
                    step: 0,
                });
                return self.message("Empty recording discarded; previous macro retained");
            }
            self.macros.last = Some(Arc::new(Definition {
                entry: recorder.entry,
                steps: recorder.steps,
            }));
        } else if command == MacroCommand::End {
            return self.message("No keyboard macro is being recorded");
        }
        let count = if recorded {
            match command {
                MacroCommand::EndOrCall => 0,
                MacroCommand::End => {
                    if argument.supplied() {
                        argument.value().saturating_sub(1)
                    } else {
                        0
                    }
                }
                _ => argument.value(),
            }
        } else {
            argument.value()
        };
        if count == 0 && recorded {
            return self.message("Keyboard macro recorded");
        }
        if count <= 0 {
            return self.message("Keyboard macro currently requires a positive replay count");
        }
        self.start_macro(count as usize)
    }

    fn start_macro(&mut self, iterations: usize) -> Action {
        let Some(definition) = self.macros.last.clone() else {
            return self.message("No keyboard macro is available");
        };
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
        self.macros.runner = Some(Runner {
            definition,
            iterations,
            iteration: 0,
            step: 0,
            dispatched: 0,
            started: Instant::now(),
            expected: (self.windows.selected_id(), self.view().point),
            waiting: false,
            group: crate::buffer::edit::AutomaticGroup::new(budget.clone()),
            budget,
        });
        self.message("Keyboard macro playing")
    }
}

fn supported(command: Command) -> bool {
    if matches!(
        command,
        Command::Navigate(crate::command::Navigation {
            target: crate::command::navigation::Target::Search(_),
            ..
        })
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
    )
}
