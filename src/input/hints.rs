use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use crossterm::event::{Event, KeyEvent};

use super::keymap::Sequence;
use super::{Input, Outcome};
use crate::command::{Command, TreeCommand};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TerminalOwner {
    id: crate::term::SessionId,
    pub(super) copy: bool,
}

impl Input {
    /// Inspect finite admitted sequences through the real resolver, without
    /// executing outcomes or borrowing the live owner's prefix/history state.
    pub(super) fn command_hints(&self) -> BTreeMap<String, String> {
        let mut sequences: BTreeSet<_> = self
            .active_map()
            .bindings()
            .map(|(sequence, _)| sequence.clone())
            .collect();
        if self.macro_editing {
            sequences.insert(Sequence::parse("C-]").expect("native macro abort key"));
        }
        if self.macro_bindings_active() {
            sequences.extend(self.macro_bindings.keys().cloned());
        }
        if self.workspace_completion && !self.workspace_startup {
            sequences.extend(
                self.keymaps
                    .map(super::keymap::Context::Completion)
                    .bindings()
                    .map(|(sequence, _)| sequence.clone()),
            );
        }
        let mut sequences: Vec<_> = sequences
            .into_iter()
            .map(|sequence| {
                let label = sequence.to_string();
                (sequence.chords().len(), label, sequence)
            })
            .collect();
        sequences.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        let mut probe = self.binding_probe();
        let mut hints = BTreeMap::new();
        for (_, label, sequence) in sequences {
            probe.cancel();
            probe.workspace_completion = self.workspace_completion;
            if let Some(command) = probe.resolve_hint(&sequence) {
                let Some(command) = self.hint_command(command) else {
                    continue;
                };
                hints.entry(command.name().to_owned()).or_insert(label);
            }
        }
        if let Some(label) = hints.get("prefix-argument").cloned() {
            hints.insert("universal-argument".into(), label);
        }
        hints
    }

    fn hint_command(&self, command: Command) -> Option<Command> {
        if self.map_context() != super::keymap::Context::Tree {
            return Some(command);
        }
        if command == Command::InsertTab {
            return None;
        }
        Some(TreeCommand::alias(command).map_or(command, Command::Tree))
    }

    fn resolve_hint(&mut self, sequence: &Sequence) -> Option<Command> {
        if !self.workspace_startup
            && let Some(owner) = self.workspace_terminal
        {
            let first = sequence.chords()[0];
            if owner.copy {
                // Copy-mode Return belongs to its terminal owner, not newline.
                if first.plain() == Some(crossterm::event::KeyCode::Enter) {
                    return None;
                }
            } else {
                if Self::terminal_copy_prefix(first) {
                    return None;
                }
                if let Some(command) = Self::terminal_host_command(first) {
                    return (sequence.chords().len() == 1).then_some(command);
                }
                if !self.terminal_host_key(first) {
                    return None;
                }
            }
        }
        self.resolve_hint_sequence(sequence)
    }

    fn resolve_hint_sequence(&mut self, sequence: &Sequence) -> Option<Command> {
        let now = Instant::now();
        let last = sequence.chords().len() - 1;
        for (index, chord) in sequence.chords().iter().enumerate() {
            match self.handle_at(Event::Key(KeyEvent::new(chord.code, chord.modifiers)), now) {
                Outcome::Command(command) | Outcome::ArgumentPrefix(command) if index == last => {
                    return (!self.workspace_startup || Self::startup_command(command))
                        .then_some(command);
                }
                Outcome::Prefix(_) | Outcome::ArgumentPrefix(_) if index < last => {}
                _ => return None,
            }
        }
        None
    }

    fn binding_probe(&self) -> Self {
        // The probe owns no persistence worker, live prompt, pending operator,
        // usage events or shell history. Only admitted maps and origin matter.
        Self {
            plugin_commands: Vec::new(),
            plugin_descriptions: Default::default(),
            macro_editing: self.macro_editing,
            logical_key: None,
            profile: self.profile,
            editing_state: self.editing_state,
            editing_owner: self.editing_owner,
            context: Default::default(),
            prefix: None,
            grammar: Default::default(),
            argument: Default::default(),
            navigation_source: None,
            chords: Vec::new(),
            keymaps: self.keymaps.clone(),
            plugin_keys: self.plugin_keys.clone(),
            workspace_tree: self.workspace_tree,
            workspace_startup: self.workspace_startup,
            workspace_bookmarks: self.workspace_bookmarks,
            workspace_macro: self.workspace_macro,
            workspace_git: self.workspace_git,
            workspace_git_commit: self.workspace_git_commit,
            workspace_git_history: self.workspace_git_history,
            workspace_git_inspect: self.workspace_git_inspect,
            workspace_git_refs: self.workspace_git_refs,
            workspace_git_sequence: self.workspace_git_sequence,
            workspace_git_worktree: self.workspace_git_worktree,
            workspace_git_stash: self.workspace_git_stash,
            workspace_git_conflict: self.workspace_git_conflict,
            workspace_completion: self.workspace_completion,
            workspace_org: self.workspace_org,
            workspace_markdown: self.workspace_markdown,
            workspace_markdown_table: self.workspace_markdown_table,
            workspace_terminal: self.workspace_terminal,
            meta_deadline: None,
            suppress_repeat: false,
            history: Default::default(),
            command_usage: Default::default(),
            shell_history: Default::default(),
            ai_commands: Vec::new(),
            macro_commands: Vec::new(),
            macro_descriptions: Default::default(),
            macro_bindings: self.macro_bindings.clone(),
            policy: self.policy.clone(),
        }
    }

    pub(crate) fn set_terminal_owner(&mut self, id: Option<crate::term::SessionId>, copy: bool) {
        let owner = id.map(|id| TerminalOwner { id, copy });
        if self.workspace_terminal != owner {
            self.clear_prefix();
            self.retire_command_hints();
            self.workspace_terminal = owner;
        }
    }

    pub(super) fn retire_command_hints(&mut self) {
        if let super::Context::Command(palette) = &mut self.context {
            palette.retire_hints();
        }
    }
}
