mod bindings;
mod inventory;
mod key;
mod meta;
mod palette;
mod paste;
mod prompt;
mod search;

use std::collections::VecDeque;
use std::fmt;
use std::time::Instant;

use crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::command::Command;
pub use inventory::Selection;
pub use key::Key;
pub use meta::META_TIMEOUT;
pub use palette::Palette;
pub use prompt::{LIMIT, Prompt};
pub use search::SearchAction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prefix {
    ControlX,
    Rectangle,
    Meta,
}

impl Prefix {
    pub fn label(self) -> &'static str {
        match self {
            Self::ControlX => "C-x",
            Self::Rectangle => "C-x r",
            Self::Meta => "ESC",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Editing,
    Search,
    Command,
    Argument,
    Decision,
    Inventory,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Search(SearchAction),
    Ignored,
    Changed,
    Complete,
    Selection(Selection),
    Prefix(Prefix),
    Command(Command),
    Editing(Key),
    Paste(String),
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
    Search(Prompt),
    Command(Palette),
    Argument(Prompt),
    Decision(&'static str),
    Inventory,
}

/// Sole keyboard focus and prefix owner. Completion consumes the prompt once;
/// opening a prompt clears pending prefixes, and cancellation restores editing.
#[derive(Debug, Default)]
pub struct Input {
    context: Context,
    prefix: Option<Prefix>,
    meta_deadline: Option<Instant>,
    suppress_repeat: bool,
    history: VecDeque<Command>,
}

impl Input {
    pub fn focus(&self) -> Focus {
        match self.context {
            Context::Editing => Focus::Editing,
            Context::Search(_) => Focus::Search,
            Context::Command(_) => Focus::Command,
            Context::Argument(_) => Focus::Argument,
            Context::Decision(_) => Focus::Decision,
            Context::Inventory => Focus::Inventory,
        }
    }

    pub fn prefix(&self) -> Option<Prefix> {
        self.meta_deadline.map(|_| Prefix::Meta).or(self.prefix)
    }

    pub fn prompt(&self) -> Option<&Prompt> {
        match &self.context {
            Context::Search(prompt) => Some(prompt),
            Context::Command(palette) => Some(palette.prompt()),
            Context::Argument(prompt) => Some(prompt),
            _ => None,
        }
    }

    pub fn label(&self) -> Option<&'static str> {
        match &self.context {
            Context::Search(prompt) => Some(prompt.label()),
            Context::Command(_) => Some("M-x "),
            Context::Argument(prompt) => Some(prompt.label()),
            Context::Decision(label) => Some(label),
            Context::Inventory => Some("Buffers: arrows move, Enter selects, C-g cancels"),
            _ => None,
        }
    }

    pub fn argument(&mut self, label: &'static str, initial: &str) -> Result<(), Error> {
        self.available()?;
        let prompt = Prompt::new(label, initial)?;
        self.context = Context::Argument(prompt);
        self.prefix = None;
        self.meta_deadline = None;
        Ok(())
    }

    pub fn decision(&mut self, label: &'static str) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Decision(label);
        self.prefix = None;
        self.meta_deadline = None;
        Ok(())
    }

    /// Paste bypasses key decoding; releases, unrelated non-key events and
    /// command/confirmation repeats are ignored.
    /// Editing outcomes are routed input, not claims of implemented commands.
    pub fn handle(&mut self, event: Event) -> Outcome {
        self.handle_at(event, Instant::now())
    }

    /// Deliver an event at a monotonic time. Expired Escape state is cleared
    /// before the event; callers pump `expire` to present idle expiration notices.
    pub fn handle_at(&mut self, event: Event, now: Instant) -> Outcome {
        self.expire(now);
        if let Event::Paste(text) = event {
            return self.paste(text);
        }
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
        self.meta_key(key, now)
    }

    fn repeat_blocked(&self, code: KeyCode) -> bool {
        self.suppress_repeat
            || self.meta_deadline.is_some()
            || self.prefix.is_some()
            || self.focus() == Focus::Decision
            || (matches!(
                self.focus(),
                Focus::Command | Focus::Argument | Focus::Inventory | Focus::Search
            ) && code == KeyCode::Enter)
    }

    pub fn cancel(&mut self) {
        self.context = Context::Editing;
        self.prefix = None;
        self.meta_deadline = None;
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
        if matches!(self.context, Context::Command(_)) {
            return self.command_key(key);
        }
        let outcome = match &mut self.context {
            Context::Search(prompt) => search::handle(prompt, key),
            Context::Editing => self.editing(key),
            Context::Argument(prompt) => prompt.handle(key),
            Context::Decision(_) => decide(key),
            Context::Inventory => inventory::handle(key),
            Context::Command(_) => unreachable!("command focus routes before general contexts"),
        };
        self.consume(outcome)
    }

    fn consume(&mut self, outcome: Outcome) -> Outcome {
        if matches!(
            outcome,
            Outcome::Argument(_)
                | Outcome::Decision(_)
                | Outcome::Selection(Selection::Accept)
                | Outcome::Search(SearchAction::Accept)
        ) {
            self.context = Context::Editing;
            self.suppress_repeat = true;
        }
        outcome
    }

    fn editing(&mut self, key: Key) -> Outcome {
        match self.prefix.take() {
            Some(Prefix::ControlX) => self.control_x(key),
            Some(Prefix::Rectangle) => {
                self.suppress_repeat = true;
                Outcome::Message("Undefined key sequence")
            }
            Some(Prefix::Meta) => unreachable!("Meta deadlines are separate from chord prefixes"),
            None => self.base(key),
        }
    }

    fn control_x(&mut self, key: Key) -> Outcome {
        self.suppress_repeat = true;
        if let Some(command) = bindings::control_x(key) {
            return Outcome::Command(command);
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
        if let Some(command) = bindings::binding(key) {
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
