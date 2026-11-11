use crate::command::{Command, CompletionCommand, PrefixArgument};
use crate::input::Outcome;
use crate::macros::{Argument, CommandStep, Role, Step, StopReason};
use crate::session::{Action, Pending, Session};

enum Capture {
    Waiting,
    Named(Command),
    Physical(Option<Command>),
}

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
        let command = outcome_command(outcome);
        if command == Some(Command::KeyboardQuit) {
            self.macros.fail(StopReason::Canceled);
            return None;
        }
        let capture = match self.macro_capture_admission(outcome, command) {
            Ok(Capture::Waiting) => return None,
            Ok(capture) => capture,
            Err(action) => return Some(action),
        };
        self.macro_capture_accepted(outcome, &capture)
    }

    fn macro_capture_admission(
        &mut self,
        outcome: &Outcome,
        command: Option<Command>,
    ) -> Result<Capture, Action> {
        let recorder = self.macros.recorder.as_mut().expect("capture owner");
        let named = recorder.selecting && matches!(self.pending, Some(Pending::Command));
        if command == Some(Command::ExecuteExtended) && !named {
            recorder.discard_pending();
            recorder.selecting = true;
            return Ok(Capture::Waiting);
        }
        if recorder.selecting && !named {
            return Ok(Capture::Waiting);
        }
        self.macro_capture_selected_outcome(outcome, command, named)
    }

    fn macro_capture_selected_outcome(
        &mut self,
        outcome: &Outcome,
        command: Option<Command>,
        named: bool,
    ) -> Result<Capture, Action> {
        if named && command.is_none() {
            if self.input.focus() == crate::input::Focus::Editing
                && matches!(outcome, Outcome::Message(_))
            {
                return Err(self.stop_macro(
                    StopReason::MissingCommand,
                    "Macro command selection was not accepted",
                ));
            }
            // Query text, completion and candidate navigation are presentation.
            return Ok(Capture::Waiting);
        }
        if self.macro_capture_control(outcome, command) {
            return Ok(Capture::Waiting);
        }
        if named {
            Ok(Capture::Named(command.expect("accepted named command")))
        } else {
            Ok(Capture::Physical(command))
        }
    }

    fn macro_capture_control(&mut self, outcome: &Outcome, command: Option<Command>) -> bool {
        let recorder = self.macros.recorder.as_mut().expect("capture owner");
        if command.is_some_and(|c| matches!(c, Command::Macro(_))) {
            recorder.selecting = false;
            recorder.discard_pending();
            return true;
        }
        if recorder.command.is_some() {
            return true;
        }
        if matches!(command, Some(Command::PluginDefined(_))) {
            // Live plugin admission owns semantic capture after validating its
            // policy and literal arguments; never retain the physical trigger.
            recorder.selecting = false;
            recorder.discard_pending();
            return true;
        }
        if let Some(command @ Command::MacroNamed(_)) = command {
            recorder.selecting = false;
            recorder.discard_pending();
            recorder.begin_command(
                CommandStep {
                    name: command.name().into(),
                    argument: outcome_argument(outcome),
                    keys: None,
                    arguments: Vec::new(),
                },
                true,
            );
            return true;
        }
        if let Some(Command::Completion(control)) = command {
            recorder.selecting = false;
            if control == CompletionCommand::AtPoint {
                recorder.discard_pending();
            }
            return true;
        }
        false
    }

    fn macro_capture_accepted(&mut self, outcome: &Outcome, capture: &Capture) -> Option<Action> {
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
        match capture {
            Capture::Named(command) => {
                self.macro_capture_named(*command, outcome_argument(outcome))
            }
            Capture::Physical(command) => self.macro_capture_physical(outcome, *command),
            Capture::Waiting => None,
        }
    }

    fn macro_capture_named(
        &mut self,
        command: Command,
        argument: PrefixArgument,
    ) -> Option<Action> {
        let command = command.canonical();
        let step = CommandStep {
            name: command.name().into(),
            argument,
            keys: None,
            arguments: Vec::new(),
        };
        let recorder = self.macros.recorder.as_mut().expect("capture owner");
        recorder.selecting = false;
        recorder.discard_pending();
        if command.prefix_control() {
            if let Err(reason) = recorder.stage(Step::Command(step)) {
                return Some(self.stop_macro(reason, "Keyboard macro recording limit exceeded"));
            }
        } else {
            recorder.begin_command(step, true);
        }
        None
    }

    fn macro_capture_physical(
        &mut self,
        outcome: &Outcome,
        command: Option<Command>,
    ) -> Option<Action> {
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
            return self.macro_capture_physical_step(step, outcome, command);
        }
        None
    }

    fn macro_capture_physical_step(
        &mut self,
        step: Step,
        outcome: &Outcome,
        command: Option<Command>,
    ) -> Option<Action> {
        let recorder = self.macros.recorder.as_mut().expect("capture owner");
        if let Err(reason) = recorder.stage(step) {
            return Some(self.stop_macro(reason, "Keyboard macro recording limit exceeded"));
        }
        if let Some(command) = command.filter(|c| prompting(*c)) {
            recorder.begin_command(
                CommandStep {
                    name: command.canonical().name().into(),
                    argument: outcome_argument(outcome),
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
        None
    }

    /// Called by the native owner only after it validates an accepted value.
    pub(in crate::session) fn macro_accept_argument(&mut self, role: Role, value: &str) -> bool {
        if self.macros.expanded {
            return true;
        }
        self.macro_recent_argument(role, value);
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
        self.macro_recent_finish();
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

pub(super) fn outcome_command(outcome: &Outcome) -> Option<Command> {
    match outcome {
        Outcome::Command(command) | Outcome::ArgumentPrefix(command) => Some(*command),
        Outcome::Invocation(invocation) => Some(invocation.command),
        _ => None,
    }
}

fn outcome_argument(outcome: &Outcome) -> PrefixArgument {
    match outcome {
        Outcome::Invocation(invocation) => invocation.argument,
        _ => PrefixArgument::default(),
    }
}
