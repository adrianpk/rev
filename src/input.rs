mod context;
mod dispatch;
mod inventory;
mod key;
pub mod keymap;
mod meta;
mod palette;
mod paste;
mod path;
mod prompt;
mod replace;
mod search;
mod tree;

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
    Tree,
    Meta,
    Sequence,
}

impl Prefix {
    pub fn label(self) -> &'static str {
        match self {
            Self::ControlX => "C-x",
            Self::Rectangle => "C-x r",
            Self::Tree => "C-x t",
            Self::Meta => "ESC",
            Self::Sequence => "Key sequence",
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
            Self::Limit => formatter.write_str("prompt input limit exceeded"),
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
    Path(Prompt),
    Literal(Prompt),
    ReplaceDecision,
    Decision(&'static str),
    Inventory,
}

/// Sole keyboard focus and prefix owner. Completion consumes the prompt once;
/// opening a prompt clears pending prefixes, and cancellation restores editing.
#[derive(Debug, Default)]
pub struct Input {
    context: Context,
    prefix: Option<Prefix>,
    chords: Vec<Key>,
    keymaps: keymap::Effective,
    workspace_tree: bool,
    meta_deadline: Option<Instant>,
    suppress_repeat: bool,
    history: VecDeque<Command>,
    policy: crate::config::Minibuffer,
}

impl Input {
    /// Uses admitted context maps and session-local prompt/history bounds.
    /// The default constructor retains the compiled preferences.
    pub fn with_settings(settings: &crate::config::Snapshot) -> Self {
        Self {
            policy: settings.settings().minibuffer.clone(),
            keymaps: settings.keymaps().clone(),
            ..Self::default()
        }
    }

    pub fn focus(&self) -> Focus {
        match self.context {
            Context::Editing => Focus::Editing,
            Context::Search(_) => Focus::Search,
            Context::Command(_) => Focus::Command,
            Context::Argument(_) | Context::Path(_) | Context::Literal(_) => Focus::Argument,
            Context::Decision(_) | Context::ReplaceDecision => Focus::Decision,
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
            Context::Argument(prompt) | Context::Path(prompt) | Context::Literal(prompt) => {
                Some(prompt)
            }
            _ => None,
        }
    }

    pub fn label(&self) -> Option<&'static str> {
        match &self.context {
            Context::Search(prompt) => Some(prompt.label()),
            Context::Command(_) => Some("M-x "),
            Context::Argument(prompt) | Context::Path(prompt) | Context::Literal(prompt) => {
                Some(prompt.label())
            }
            Context::ReplaceDecision => Some(replace::QUESTION),
            Context::Decision(label) => Some(label),
            Context::Inventory => Some("Buffers: arrows move, Enter selects, C-g cancels"),
            _ => None,
        }
    }

    pub fn argument(&mut self, label: &'static str, initial: &str) -> Result<(), Error> {
        self.available()?;
        let prompt = Prompt::new(label, initial, self.policy.input_limit)?;
        self.context = Context::Argument(prompt);
        self.clear_prefix();
        Ok(())
    }

    pub fn decision(&mut self, label: &'static str) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Decision(label);
        self.clear_prefix();
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
        let Some(key) = Key::normalize(event) else {
            return Outcome::Ignored;
        };
        if event.kind == KeyEventKind::Repeat
            && (self.repeat_blocked(key.code) || !self.repeatable(key))
        {
            return Outcome::Ignored;
        }
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
        self.clear_prefix();
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
        if self.focus() == Focus::Decision {
            return self.decision_key(key);
        }
        self.mapped(key)
    }

    fn focused(&mut self, key: Key) -> Outcome {
        match &mut self.context {
            Context::Search(prompt) => search::handle(prompt, key),
            Context::Editing => self.literal(key),
            Context::Argument(prompt) => prompt.handle(key),
            Context::Path(prompt) => path::handle(prompt, key),
            Context::Literal(prompt) => replace::argument(prompt, key),
            Context::ReplaceDecision => replace::answer(key),
            Context::Decision(_) => decide(key),
            Context::Inventory => inventory::handle(key),
            Context::Command(_) => self.command_key(key),
        }
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
            self.clear_prefix();
            self.suppress_repeat = true;
        }
        outcome
    }

    fn literal(&self, key: Key) -> Outcome {
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
