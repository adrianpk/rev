use crossterm::event::{KeyCode, KeyModifiers};

use crate::command::{Binding, Command, Motion, PromptAction, TreeCommand};

use super::keymap::{Context as MapContext, Sequence};
use super::{Focus, Input, Key, Outcome};

impl Input {
    pub(super) fn map_context(&self) -> MapContext {
        if matches!(self.context, super::Context::Calendar(_)) {
            return MapContext::OrgCalendar;
        }
        if self.focus() != Focus::Editing {
            return MapContext::Minibuffer;
        }
        if self.workspace_tree {
            return MapContext::Tree;
        }
        self.workspace_org
            .map_or(MapContext::Editor, super::OrgContext::keymap)
    }

    pub(super) fn context_binding(&mut self, binding: Binding, key: Key, custom: bool) -> Outcome {
        if binding == Binding::Command(Command::KeyboardQuit) {
            self.cancel();
            return Outcome::Command(Command::KeyboardQuit);
        }
        if matches!(self.context, super::Context::Calendar(_)) {
            if let Binding::Command(command @ Command::Org(c)) = binding
                && c.calendar()
            {
                return Outcome::Command(command);
            }
            if !MapContext::Minibuffer.available(binding) {
                return Outcome::Message("Calendar prompt owns input focus");
            }
        }
        if self.focus() == Focus::Editing {
            if self.workspace_org == Some(super::OrgContext::Calendar)
                && !MapContext::OrgCalendar.local_available(binding)
            {
                return Outcome::Message("Prompt action requires an active prompt");
            }
            let Binding::Command(command) = binding else {
                return Outcome::Message("Prompt action requires an active prompt");
            };
            return Outcome::Command(command);
        }
        // Existing owners retain completion, history, literal search tabs and
        // argument consumption. Inherited keys keep their audited stage limits
        // (for example inventory arrows versus command-candidate C-p/C-n).
        let key = if custom { prompt_key(binding) } else { key };
        let outcome = self.focused(key);
        self.consume(outcome)
    }

    pub(super) fn binding_repeats(&self, binding: Binding) -> bool {
        if self.map_context() == MapContext::Tree {
            return tree_repeats(binding);
        }
        match binding {
            Binding::Command(command) => command.repeatable(),
            Binding::Prompt(action) => action != PromptAction::Accept,
        }
    }

    pub(super) fn decision_key(&mut self, key: Key) -> Outcome {
        // Decisions accept only native explicit answers, plus cancellation.
        // A failed cancel prefix consumes its suffix without interpreting y/n.
        let continued = !self.chords.is_empty();
        self.chords.push(key);
        let Some(sequence) = Sequence::from_chords(&self.chords) else {
            return self.unmatched(key, continued);
        };
        let map = self.keymaps.map(MapContext::Minibuffer);
        if map.lookup(&sequence) == Some(Binding::Command(Command::KeyboardQuit)) {
            self.cancel();
            return Outcome::Command(Command::KeyboardQuit);
        }
        if map.is_cancel_prefix(&sequence) {
            return self.wait_sequence(&sequence);
        }
        self.unmatched(key, continued)
    }
}

fn tree_repeats(binding: Binding) -> bool {
    match binding {
        Binding::Command(Command::Tree(command)) => matches!(
            command,
            TreeCommand::Previous | TreeCommand::Next | TreeCommand::Expand | TreeCommand::Collapse
        ),
        Binding::Command(Command::Motion(_) | Command::Scroll(_)) => true,
        _ => false,
    }
}

/// Translate a validated prompt target into the existing owner's canonical
/// operation. This does not synthesize another event or re-enter key lookup.
fn prompt_key(binding: Binding) -> Key {
    let code = match binding {
        Binding::Prompt(PromptAction::Accept) => KeyCode::Enter,
        Binding::Prompt(PromptAction::Complete) => KeyCode::Tab,
        Binding::Prompt(PromptAction::Previous) => KeyCode::Up,
        Binding::Prompt(PromptAction::Next) => KeyCode::Down,
        Binding::Prompt(PromptAction::HistoryPrevious) => return history_key('p'),
        Binding::Prompt(PromptAction::HistoryNext) => return history_key('n'),
        Binding::Command(Command::BackwardDeleteChar) => KeyCode::Backspace,
        Binding::Command(Command::DeleteChar) => KeyCode::Delete,
        Binding::Command(Command::Motion(motion)) => motion_code(motion),
        Binding::Command(command) => return search_key(command),
    };
    Key {
        code,
        modifiers: KeyModifiers::NONE,
    }
}

fn history_key(character: char) -> Key {
    Key {
        code: KeyCode::Char(character),
        modifiers: KeyModifiers::ALT,
    }
}

fn motion_code(motion: Motion) -> KeyCode {
    match motion {
        Motion::BackwardChar => KeyCode::Left,
        Motion::ForwardChar => KeyCode::Right,
        Motion::LineStart => KeyCode::Home,
        Motion::LineEnd => KeyCode::End,
        _ => unreachable!("minibuffer maps admit character and line motion only"),
    }
}

fn search_key(command: Command) -> Key {
    let character = match command {
        Command::SearchForward => 's',
        Command::SearchBackward => 'r',
        _ => unreachable!("validated remaining prompt targets are search actions"),
    };
    Key {
        code: KeyCode::Char(character),
        modifiers: KeyModifiers::CONTROL,
    }
}
