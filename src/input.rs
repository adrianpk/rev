mod argument;
pub use argument::{Invocation, InvocationOrigin, TextInvocation};
mod completion;
mod context;
mod dispatch;
mod grammar;
mod hints;
mod inventory;
mod key;
pub mod keymap;
mod macros;
mod meta;
mod modal;
pub use macros::MacroQuery;
pub use modal::EditingState;
mod names;
pub(crate) use names::Names;
mod org;
mod palette;
mod paste;
mod path;
mod prompt;
mod replace;
mod search;
mod shell;
mod tree;
mod usage;

use std::collections::VecDeque;
use std::fmt;
use std::time::Instant;

use crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::command::Command;
pub use inventory::Selection;
pub use key::Key;
pub use meta::META_TIMEOUT;
pub use org::OrgContext;
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
    Pointer(crate::pointer::Event),
    Search(SearchAction),
    Ignored,
    Changed,
    Complete,
    Selection(Selection),
    Prefix(Prefix),
    Command(Command),
    Invocation(Box<Invocation>),
    TextInvocation(Box<TextInvocation>),
    ArgumentPrefix(Command),
    Editing(Key),
    Paste(String),
    Argument(String),
    PathAccept,
    Decision(bool),
    MacroQuery(MacroQuery),
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
    ArgumentDigit(Prompt),
    Shell(Prompt),
    Names(Names),
    Calendar(Prompt),
    Path(Prompt),
    Literal(Prompt),
    ReplaceDecision,
    MacroQuery,
    Decision(&'static str),
    Inventory,
}

/// Sole keyboard focus and prefix owner. Completion consumes the prompt once;
/// opening a prompt clears pending prefixes, and cancellation restores editing.
#[derive(Debug, Default)]
pub struct Input {
    pub(crate) logical_key: Option<Key>,
    pub(crate) macro_editing: bool,
    profile: crate::config::InteractionProfile,
    editing_state: EditingState,
    editing_owner: Option<(crate::window::ViewId, crate::buffer::BufferId)>,
    context: Context,
    prefix: Option<Prefix>,
    grammar: grammar::Grammar,
    argument: argument::State,
    navigation_source: Option<(crate::window::ViewId, crate::buffer::edit::Point)>,
    chords: Vec<Key>,
    keymaps: keymap::Effective,
    workspace_tree: bool,
    workspace_startup: bool,
    workspace_bookmarks: bool,
    workspace_git: bool,
    workspace_git_commit: bool,
    workspace_git_history: bool,
    workspace_git_inspect: bool,
    workspace_git_refs: bool,
    workspace_git_sequence: bool,
    workspace_git_worktree: bool,
    workspace_git_stash: bool,
    workspace_git_conflict: bool,
    workspace_completion: bool,
    workspace_org: Option<OrgContext>,
    workspace_terminal: Option<hints::TerminalOwner>,
    meta_deadline: Option<Instant>,
    suppress_repeat: bool,
    history: VecDeque<Command>,
    command_usage: crate::command::usage::History,
    shell_history: shell::History,
    ai_commands: Vec<Command>,
    policy: crate::config::Minibuffer,
}

impl Input {
    /// The same reserved interrupt as route(), without consuming queued input.
    pub(crate) fn interrupt(event: &Event) -> bool {
        matches!(event, Event::Key(key) if key.kind == KeyEventKind::Press
            && Key::normalize(*key).is_some_and(|key| key.control('g')))
    }

    /// Uses admitted context maps and session-local prompt/history bounds.
    /// The default constructor retains the compiled preferences.
    pub fn with_settings(settings: &crate::config::Snapshot) -> Self {
        Self {
            ai_commands: settings
                .settings()
                .ai
                .commands
                .keys()
                .map(|key| {
                    Command::AiDefined(crate::command::AiName::new(key).expect("admitted AI name"))
                })
                .collect(),
            profile: settings.settings().editor.interaction_profile,
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
            Context::Decision(_) | Context::ReplaceDecision | Context::MacroQuery => {
                Focus::Decision
            }
            Context::Inventory => Focus::Inventory,
            _ => Focus::Argument,
        }
    }

    pub fn prefix(&self) -> Option<Prefix> {
        self.meta_deadline.map(|_| Prefix::Meta).or(self.prefix)
    }

    pub fn prompt(&self) -> Option<&Prompt> {
        match &self.context {
            Context::Search(prompt) => Some(prompt),
            Context::Command(palette) => Some(palette.prompt()),
            Context::Names(names) => Some(names.prompt()),
            Context::Calendar(prompt)
            | Context::Argument(prompt)
            | Context::ArgumentDigit(prompt)
            | Context::Shell(prompt)
            | Context::Path(prompt)
            | Context::Literal(prompt) => Some(prompt),
            _ => None,
        }
    }

    pub fn label(&self) -> Option<&str> {
        match &self.context {
            Context::Command(_) => Some("M-x "),
            Context::ReplaceDecision => Some(replace::QUESTION),
            Context::MacroQuery => Some(macros::QUESTION),
            Context::Decision(label) => Some(label),
            Context::Inventory => Some("Buffers: arrows move, Enter selects, C-g cancels"),
            _ => self.prompt().map(Prompt::label),
        }
    }

    pub fn argument(&mut self, label: &str, initial: &str) -> Result<(), Error> {
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
    /// before the event; callers must dispatch `expire` first so cancellation
    /// reaches the focused owner before fresh input, as well as during idle polls.
    pub fn handle_at(&mut self, event: Event, now: Instant) -> Outcome {
        self.logical_key = None;
        self.expire(now);
        if let Event::Mouse(event) = event {
            let outcome = self.pointer(event);
            return self.argument_outcome(outcome);
        }
        if let Event::Paste(text) = event {
            let outcome = self.paste(text);
            return self.argument_outcome(outcome);
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
        if let Some(outcome) = self.completion_key(key) {
            return outcome;
        }
        let outcome = self.meta_key(key, now);
        self.argument_outcome(outcome)
    }

    fn pointer(&self, event: crossterm::event::MouseEvent) -> Outcome {
        if self.focus() != Focus::Editing {
            return Outcome::Ignored;
        }
        crate::pointer::Event::from_mouse(event).map_or(Outcome::Ignored, Outcome::Pointer)
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
        self.logical_key = Some(key);
        if key.control('g') {
            self.cancel();
            return Outcome::Command(Command::KeyboardQuit);
        }
        if self.macro_editing && self.focus() == Focus::Editing && key.control(']') {
            self.clear_prefix();
            return Outcome::Command(Command::Macro(crate::command::MacroCommand::Abort));
        }
        if self.focus() == Focus::Decision {
            return self.decision_key(key);
        }
        self.mapped(key)
    }

    fn focused(&mut self, key: Key) -> Outcome {
        match &mut self.context {
            Context::Search(prompt) => search::handle(prompt, key),
            Context::Editing if self.workspace_startup => inventory::handle(key),
            Context::Editing => self.literal(key),
            Context::Argument(prompt) | Context::Calendar(prompt) => prompt.handle(key),
            Context::ArgumentDigit(_) => self.argument_digit(key),
            Context::Shell(prompt) => self.shell_history.handle(prompt, key),
            Context::Path(prompt) => path::handle(prompt, key),
            Context::Literal(prompt) => replace::argument(prompt, key),
            Context::ReplaceDecision => replace::answer(key),
            Context::MacroQuery => macros::answer(key),
            Context::Decision(_) => decide(key),
            Context::Inventory => inventory::handle(key),
            _ => self.chooser_key(key),
        }
    }

    fn chooser_key(&mut self, key: Key) -> Outcome {
        match self.context {
            Context::Command(_) => self.command_key(key),
            Context::Names(_) => self.names_key(key),
            _ => unreachable!("chooser input context"),
        }
    }

    fn consume(&mut self, outcome: Outcome) -> Outcome {
        if matches!(
            outcome,
            Outcome::Argument(_)
                | Outcome::Decision(_)
                | Outcome::MacroQuery(_)
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
        if self.modal_editing() && !self.editing_state.text_entry() {
            return Outcome::Message(if self.editing_state.visual().is_some() {
                "Unbound Visual key"
            } else {
                "Unbound Normal key"
            });
        }
        if !self.accepts_text() {
            if self.workspace_git {
                return Outcome::Message("Git presentation is read-only");
            }
            return Outcome::Message("Unbound Org context key");
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

impl Input {
    pub(crate) fn calendar_argument(&mut self, initial: &str) -> Result<(), Error> {
        self.available()?;
        self.context = Context::Calendar(Prompt::new(
            "Org date: ",
            initial,
            self.policy.input_limit.min(514),
        )?);
        self.clear_prefix();
        Ok(())
    }
}
