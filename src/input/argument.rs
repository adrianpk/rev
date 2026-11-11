use crossterm::event::{KeyCode, KeyModifiers};

use crate::buffer::edit::Point;
use crate::command::{ArgumentForm, Command, PrefixArgument};
use crate::window::ViewId;

use super::keymap::{Context as MapContext, Sequence};
use super::{Context, Focus, Input, Key, Outcome, Prompt};

/// Origin captured before command selection borrows input focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvocationOrigin {
    pub context: MapContext,
    pub source: Option<(ViewId, Point)>,
}

/// One resolved command and its checked input, also available to native callers
/// that retain invocations. This value has no serialized or persistent form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invocation {
    pub command: Command,
    pub argument: PrefixArgument,
    pub origin: InvocationOrigin,
}

/// One admitted source-text entry and its transient editor argument.
#[derive(Debug, PartialEq, Eq)]
pub struct TextInvocation {
    pub text: String,
    pub argument: PrefixArgument,
    pub origin: InvocationOrigin,
}

#[derive(Debug, Default)]
pub(super) struct State {
    value: PrefixArgument,
    terminated: bool,
    modal_conflict: bool,
    origin: Option<InvocationOrigin>,
    continuation: Option<(Vec<Key>, usize)>,
}

impl Input {
    pub(crate) fn invocation_origin(&self) -> InvocationOrigin {
        InvocationOrigin {
            context: self.map_context(),
            source: self.navigation_source,
        }
    }

    pub fn prefix_argument(&self) -> PrefixArgument {
        self.argument.value
    }

    pub(super) fn clear_argument(&mut self) {
        self.argument = State::default();
    }

    pub(super) fn argument_label(&self) -> Option<String> {
        let value = self.argument.value;
        if !value.supplied() {
            return None;
        }
        let form = match value.form() {
            ArgumentForm::PlainPrefix { depth } => format!("C-u depth {depth}"),
            ArgumentForm::NegativeOnly => "negative-only".into(),
            _ => "integer".into(),
        };
        let suffix = self
            .argument
            .continuation
            .as_ref()
            .and_then(|(keys, _)| Sequence::from_chords(keys))
            .or_else(|| Sequence::from_chords(&self.chords))
            .map(|sequence| format!(" {sequence}"))
            .unwrap_or_default();
        Some(format!("Argument {} ({form}){suffix}", value.value()))
    }

    pub(crate) fn prefix_control(&mut self, command: Command, key: Option<Key>) -> Outcome {
        if self.focus() != Focus::Editing || self.workspace_terminal.is_some_and(|o| !o.copy) {
            self.clear_argument();
            return Outcome::Message("Prefix argument requires editor input focus");
        }
        if self.grammar.count.is_some()
            || self.grammar.operator.is_some()
            || self.argument.modal_conflict
        {
            self.clear_prefix();
            return Outcome::Message("Editor prefix conflicts with modal count/operator");
        }
        let command = command.canonical();
        if self.argument.origin.is_none() {
            self.argument.origin = Some(InvocationOrigin {
                context: self.map_context(),
                source: self.navigation_source,
            });
        }
        let continuation = Sequence::from_chords(&self.chords)
            .filter(|sequence| self.active_map().is_prefix(sequence))
            .map(|_| (self.chords.clone(), self.chords.len()));
        self.clear_sequence();
        self.argument.continuation = None;
        self.update_argument(command, key, continuation)
    }

    fn update_argument(
        &mut self,
        command: Command,
        key: Option<Key>,
        continuation: Option<(Vec<Key>, usize)>,
    ) -> Outcome {
        let value = match command {
            Command::PrefixArgument => match self.argument.value.form() {
                ArgumentForm::Integer(_) | ArgumentForm::NegativeOnly => {
                    self.argument.terminated = true;
                    return Outcome::ArgumentPrefix(command);
                }
                _ => {
                    self.argument.continuation = continuation;
                    self.argument.value.extend()
                }
            },
            Command::NegativeArgument => {
                self.argument.terminated = false;
                Some(self.argument.value.negate())
            }
            Command::DigitArgument => {
                let digit = key.and_then(digit);
                let Some(digit) = digit else {
                    self.context = Context::ArgumentDigit(
                        Prompt::new("Digit argument: ", "", 1).expect("fixed digit prompt"),
                    );
                    return Outcome::ArgumentPrefix(command);
                };
                self.argument.terminated = false;
                self.argument.value.digit(digit)
            }
            _ => unreachable!("prefix helper resolved by registry"),
        };
        self.admit_argument(value, command)
    }

    fn admit_argument(&mut self, value: Option<PrefixArgument>, command: Command) -> Outcome {
        let Some(value) = value else {
            self.clear_prefix();
            return Outcome::Message("Prefix argument exceeds magnitude 1000000");
        };
        self.argument.value = value;
        self.suppress_repeat = true;
        Outcome::ArgumentPrefix(command)
    }

    pub(super) fn argument_entry(&mut self, key: Key) -> Option<Outcome> {
        if self.focus() != Focus::Editing || !self.argument.value.supplied() {
            return None;
        }
        if let Some(outcome) = self.argument_continuation(key) {
            return Some(outcome);
        }
        if self.argument.terminated || !self.chords.is_empty() {
            return None;
        }
        if let Some(digit) = digit(key) {
            return Some(
                self.admit_argument(self.argument.value.digit(digit), Command::DigitArgument),
            );
        }
        if key.code == KeyCode::Char('-')
            && matches!(key.modifiers, KeyModifiers::NONE | KeyModifiers::ALT)
        {
            return Some(self.prefix_control(Command::NegativeArgument, Some(key)));
        }
        None
    }

    fn argument_continuation(&mut self, key: Key) -> Option<Outcome> {
        if let Some((mut keys, root)) = self.argument.continuation.take() {
            keys.push(key);
            if let Some(sequence) = Sequence::from_chords(&keys) {
                if let Some(binding) = self.active_map().lookup(&sequence) {
                    let custom = self.active_map().is_custom(&sequence);
                    self.clear_argument();
                    return Some(self.bound(binding, key, custom, true));
                }
                if self.active_map().is_prefix(&sequence) {
                    self.argument.continuation = Some((keys, root));
                    return Some(Outcome::ArgumentPrefix(Command::PrefixArgument));
                }
            }
            if keys.len() > root + 1 {
                self.clear_prefix();
                return Some(Outcome::Message("Undefined key sequence"));
            }
        }
        None
    }

    pub(super) fn argument_digit(&mut self, key: Key) -> Outcome {
        let Context::ArgumentDigit(prompt) = &mut self.context else {
            unreachable!("digit prompt owns focus");
        };
        match prompt.handle(key) {
            Outcome::Argument(text) => {
                self.context = Context::Editing;
                self.clear_sequence();
                if let Some(value) = text.chars().next().filter(char::is_ascii_digit) {
                    self.argument.terminated = false;
                    self.admit_argument(
                        self.argument
                            .value
                            .digit(value.to_digit(10).expect("ASCII digit")),
                        Command::DigitArgument,
                    )
                } else {
                    self.clear_prefix();
                    Outcome::Message("Argument requires one decimal digit")
                }
            }
            outcome => outcome,
        }
    }

    pub(super) fn argument_outcome(&mut self, outcome: Outcome) -> Outcome {
        match outcome {
            Outcome::Command(command) => self.command_argument_outcome(command),
            Outcome::Editing(key) if self.literal_argument_key(key) => {
                let KeyCode::Char(character) = key.code else {
                    unreachable!()
                };
                self.text_invocation(character.to_string())
            }
            Outcome::Paste(text) if self.argument_admits_text() => self.text_invocation(text),
            _ => self.retire_editing_argument(outcome),
        }
    }

    fn command_argument_outcome(&mut self, command: Command) -> Outcome {
        if command.prefix_control() && self.focus() == Focus::Editing {
            return self.prefix_control(command, None);
        }
        if command == Command::ExecuteExtended {
            return Outcome::Command(command);
        }
        if self.argument.value.supplied() {
            let argument = std::mem::take(&mut self.argument);
            return Outcome::Invocation(Box::new(Invocation {
                command,
                argument: argument.value,
                origin: argument.origin.expect("supplied input has an origin"),
            }));
        }
        self.clear_argument();
        Outcome::Command(command)
    }

    fn argument_admits_text(&self) -> bool {
        self.argument.value.supplied() && self.focus() == Focus::Editing
    }

    fn literal_argument_key(&self, key: Key) -> bool {
        self.argument_admits_text()
            && matches!(key.plain(), Some(KeyCode::Char(c)) if !c.is_control())
    }

    fn retire_editing_argument(&mut self, outcome: Outcome) -> Outcome {
        match outcome {
            Outcome::Editing(_) | Outcome::Paste(_) | Outcome::Pointer(_) | Outcome::Message(_)
                if self.focus() == Focus::Editing =>
            {
                self.clear_argument();
                outcome
            }
            _ => outcome,
        }
    }

    fn text_invocation(&mut self, text: String) -> Outcome {
        let argument = std::mem::take(&mut self.argument);
        Outcome::TextInvocation(Box::new(TextInvocation {
            text,
            argument: argument.value,
            origin: argument.origin.expect("supplied text has an origin"),
        }))
    }

    pub(super) fn retain_modal_argument_conflict(&mut self, conflict: bool) {
        self.argument.modal_conflict = conflict;
    }

    pub(crate) fn invocation_current(&self, origin: InvocationOrigin) -> bool {
        self.focus() == Focus::Editing
            && self.map_context() == origin.context
            && self.navigation_source == origin.source
    }
}

fn digit(key: Key) -> Option<u32> {
    if !matches!(key.modifiers, KeyModifiers::NONE | KeyModifiers::ALT) {
        return None;
    }
    match key.code {
        KeyCode::Char(value) if value.is_ascii_digit() => value.to_digit(10),
        _ => None,
    }
}
