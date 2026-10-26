mod key;
mod prompt;

use std::fmt;

use crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::command::Command;
pub use key::Key;
pub use prompt::{LIMIT, Prompt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prefix {
    ControlX,
    Rectangle,
}

impl Prefix {
    pub fn label(self) -> &'static str {
        match self {
            Self::ControlX => "C-x",
            Self::Rectangle => "C-x r",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Editing,
    Argument,
    Decision,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Ignored,
    Prefix(Prefix),
    Command(Command),
    Editing(Key),
    Argument(String),
    Decision(bool),
    Message(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Busy,
    Limit,
    Control,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Busy => formatter.write_str("a prompt already owns input focus"),
            Self::Limit => formatter.write_str("prompt limit is 4096 characters"),
            Self::Control => formatter.write_str("prompt initial text contains control characters"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Default)]
enum Context {
    #[default]
    Editing,
    Argument(Prompt),
    Decision(&'static str),
}

/// Sole keyboard focus and prefix owner. Completion consumes the prompt once;
/// opening a prompt clears pending prefixes, and cancellation restores editing.
#[derive(Debug, Default)]
pub struct Input {
    context: Context,
    prefix: Option<Prefix>,
    suppress_repeat: bool,
}

impl Input {
    pub fn focus(&self) -> Focus {
        match self.context {
            Context::Editing => Focus::Editing,
            Context::Argument(_) => Focus::Argument,
            Context::Decision(_) => Focus::Decision,
        }
    }

    pub fn prefix(&self) -> Option<Prefix> {
        self.prefix
    }

    pub fn prompt(&self) -> Option<&Prompt> {
        match &self.context {
            Context::Argument(prompt) => Some(prompt),
            _ => None,
        }
    }

    pub fn label(&self) -> Option<&'static str> {
        match &self.context {
            Context::Argument(prompt) => Some(prompt.label()),
            Context::Decision(label) => Some(label),
            _ => None,
        }
    }

    pub fn argument(&mut self, label: &'static str, initial: &str) -> Result<(), Error> {
        self.available()?;
        let prompt = Prompt::new(label, initial)?;
        self.context = Context::Argument(prompt);
        self.prefix = None;
        Ok(())
    }

    pub fn decision(&mut self, label: &'static str) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Decision(label);
        self.prefix = None;
        Ok(())
    }

    /// Non-key events, releases, and command/confirmation repeats are ignored.
    /// Editing outcomes are routed input, not claims of implemented commands.
    pub fn handle(&mut self, event: Event) -> Outcome {
        let Event::Key(event) = event else {
            return Outcome::Ignored;
        };
        if event.kind == KeyEventKind::Press {
            self.suppress_repeat = false;
        }
        if event.kind == KeyEventKind::Repeat && self.repeat_blocked(event.code) {
            return Outcome::Ignored;
        }
        let Some(key) = Key::normalize(event) else {
            return Outcome::Ignored;
        };
        self.route(key)
    }

    fn repeat_blocked(&self, code: KeyCode) -> bool {
        self.suppress_repeat
            || self.prefix.is_some()
            || self.focus() == Focus::Decision
            || (self.focus() == Focus::Argument && code == KeyCode::Enter)
    }

    pub fn cancel(&mut self) {
        self.context = Context::Editing;
        self.prefix = None;
        self.suppress_repeat = true;
    }

    fn available(&self) -> Result<(), Error> {
        if self.focus() != Focus::Editing {
            return Err(Error::Busy);
        }
        Ok(())
    }

    fn route(&mut self, key: Key) -> Outcome {
        if key.control('g') {
            self.cancel();
            return Outcome::Command(Command::KeyboardQuit);
        }
        let outcome = match &mut self.context {
            Context::Editing => self.editing(key),
            Context::Argument(prompt) => prompt.handle(key),
            Context::Decision(_) => decide(key),
        };
        if matches!(outcome, Outcome::Argument(_) | Outcome::Decision(_)) {
            self.context = Context::Editing;
            self.suppress_repeat = true;
        }
        outcome
    }

    fn editing(&mut self, key: Key) -> Outcome {
        match self.prefix.take() {
            Some(Prefix::ControlX) => self.control_x(key),
            Some(Prefix::Rectangle) => Outcome::Message("Undefined key sequence"),
            None => self.base(key),
        }
    }

    fn control_x(&mut self, key: Key) -> Outcome {
        if key.control('c') {
            return Outcome::Command(Command::Quit);
        }
        if key.plain() == Some(KeyCode::Char('r')) {
            return self.wait(Prefix::Rectangle);
        }
        Outcome::Message("Undefined key sequence")
    }

    fn base(&mut self, key: Key) -> Outcome {
        if key.control('x') {
            return self.wait(Prefix::ControlX);
        }
        if let Some(command) = binding(key) {
            return Outcome::Command(command);
        }
        if key.plain().is_none() {
            return Outcome::Message("Unbound key");
        }
        if matches!(key.code, KeyCode::Char(character) if character.is_control()) {
            return Outcome::Message("Control character rejected");
        }
        Outcome::Editing(key)
    }

    fn wait(&mut self, prefix: Prefix) -> Outcome {
        self.prefix = Some(prefix);
        Outcome::Prefix(prefix)
    }
}

fn decide(key: Key) -> Outcome {
    match key.plain() {
        Some(KeyCode::Char('y' | 'Y')) => Outcome::Decision(true),
        Some(KeyCode::Char('n' | 'N')) => Outcome::Decision(false),
        _ => Outcome::Message("Answer y or n"),
    }
}

fn binding(key: Key) -> Option<Command> {
    if key.control('d') {
        return Some(Command::DeleteChar);
    }
    match key.plain()? {
        KeyCode::Enter => Some(Command::Newline),
        KeyCode::Tab => Some(Command::InsertTab),
        KeyCode::Backspace => Some(Command::BackwardDeleteChar),
        KeyCode::Delete => Some(Command::DeleteChar),
        _ => None,
    }
}
