use crate::command::{Command, CompletionCommand, PrefixArgument};
use crate::input::Outcome;
use crate::macros::{Argument, CommandStep, Role, Step, StopReason};
use crate::session::{Action, Pending, Session};

impl Session {
    pub(in crate::session) fn macro_capture_outcome(
        &mut self,
        outcome: &Outcome,
    ) -> Option<Action> {
        if self.macros.expanded || self.macros.recorder.is_none() {
            return None;
        }
        if matches!(self.pending, Some(Pending::MacroCounter { .. })) {
            return None;
        }
        let command = match outcome {
            Outcome::Command(command) | Outcome::ArgumentPrefix(command) => Some(*command),
            Outcome::Invocation(invocation) => Some(invocation.command),
            _ => None,
        };
        if command == Some(Command::KeyboardQuit) {
            self.macros.fail(StopReason::Canceled);
            return None;
        }
        let recorder = self.macros.recorder.as_mut().expect("capture owner");
        let named = recorder.selecting && matches!(self.pending, Some(Pending::Command));
        if command == Some(Command::ExecuteExtended) && !named {
            recorder.discard_pending();
            recorder.selecting = true;
            return None;
        }
        if recorder.selecting && !named {
            return None;
        }
        if named && command.is_none() {
            if self.input.focus() == crate::input::Focus::Editing
                && matches!(outcome, Outcome::Message(_))
            {
                return Some(self.stop_macro(
                    StopReason::MissingCommand,
                    "Macro command selection was not accepted",
                ));
            }
            // Query text, completion and candidate navigation are presentation.
            return None;
        }
        if command.is_some_and(|c| matches!(c, Command::Macro(_))) {
            recorder.selecting = false;
            recorder.discard_pending();
            return None;
        }
        if recorder.command.is_some() {
            return None;
        }
        if let Some(command @ Command::MacroNamed(_)) = command {
            let argument = match outcome {
                Outcome::Invocation(invocation) => invocation.argument,
                _ => PrefixArgument::default(),
            };
            recorder.selecting = false;
            recorder.discard_pending();
            recorder.begin_command(
                CommandStep {
                    name: command.name().into(),
                    argument,
                    keys: None,
                    arguments: Vec::new(),
                },
                true,
            );
            return None;
        }
        if let Some(Command::Completion(control)) = command {
            recorder.selecting = false;
            if control == CompletionCommand::AtPoint {
                recorder.discard_pending();
            }
            return None;
        }
        if matches!(outcome, Outcome::Ignored) {
            return None;
        }
        if matches!(outcome, Outcome::Message(_) | Outcome::Pointer(_)) {
            return Some(self.stop_macro(
                StopReason::Unsupported,
                "Keyboard macro input is unsupported",
            ));
        }
        if matches!(outcome, Outcome::Paste(text) if text.is_empty()) {
            return None;
        }
        if matches!(outcome, Outcome::Paste(text) if text.contains('\0')) {
            return Some(
                self.stop_macro(StopReason::Unsupported, "Keyboard macro paste was rejected"),
            );
        }
        let argument = match outcome {
            Outcome::Invocation(invocation) => invocation.argument,
            _ => PrefixArgument::default(),
        };
        if named {
            let command = command.expect("accepted named command").canonical();
            let step = CommandStep {
                name: command.name().into(),
                argument,
                keys: None,
                arguments: Vec::new(),
            };
            recorder.selecting = false;
            recorder.discard_pending();
            if command.prefix_control() {
                if let Err(reason) = recorder.stage(Step::Command(step)) {
                    return Some(
                        self.stop_macro(reason, "Keyboard macro recording limit exceeded"),
                    );
                }
            } else {
                recorder.begin_command(step, true);
            }
            return None;
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
            if let Err(reason) = recorder.stage(step) {
                return Some(self.stop_macro(reason, "Keyboard macro recording limit exceeded"));
            }
            if let Some(command) = command.filter(|c| prompting(*c)) {
                recorder.begin_command(
                    CommandStep {
                        name: command.canonical().name().into(),
                        argument,
                        keys: None,
                        arguments: Vec::new(),
                    },
                    false,
                );
            } else if !matches!(
                outcome,
                Outcome::Prefix(_) | Outcome::ArgumentPrefix(_) | Outcome::Changed
            ) && let Err(reason) = recorder.accept()
            {
                return Some(self.stop_macro(reason, "Keyboard macro recording limit exceeded"));
            }
        }
        None
    }

    /// Called by the native owner only after it validates an accepted value.
    pub(in crate::session) fn macro_accept_argument(&mut self, role: Role, value: &str) -> bool {
        if self.macros.expanded {
            return true;
        }
        if let Some(recorder) = self.macros.recorder.as_mut()
            && let Err(reason) = recorder.argument(Argument {
                role,
                value: value.into(),
            })
        {
            self.stop_macro(reason, "Keyboard macro argument cannot be recorded");
            return false;
        }
        true
    }

    pub(in crate::session) fn macro_finish_capture(&mut self) {
        if self.macros.recorder.is_some()
            && let Err(reason) = self.macros.admit_memory(None)
        {
            self.stop_macro(reason, "Macro retained memory exceeds 48 MiB");
            return;
        }
        if self.pending.is_none()
            && !self.layout_active()
            && self.input.focus() == crate::input::Focus::Editing
            && let Some(recorder) = self.macros.recorder.as_mut()
            && let Err(reason) = recorder.finish_command()
        {
            self.stop_macro(reason, "Keyboard macro recording limit exceeded");
        }
    }
}

fn prompting(command: Command) -> bool {
    matches!(
        command,
        Command::SearchForward
            | Command::SearchBackward
            | Command::QueryReplace
            | Command::FindFile
            | Command::GotoLine
            | Command::SaveBuffer
            | Command::WriteFile
            | Command::Buffer(_)
            | Command::Navigate(crate::command::Navigation {
                target: crate::command::navigation::Target::Search(_),
                ..
            })
            | Command::Edit(crate::command::Editing {
                operand: Some(crate::command::editing::Operand::Motion(
                    crate::command::Navigation {
                        target: crate::command::navigation::Target::Search(_),
                        ..
                    }
                )),
                ..
            })
    )
}
