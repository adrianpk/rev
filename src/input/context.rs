use crossterm::event::{KeyCode, KeyModifiers};

use crate::command::{Binding, Command, Motion, PromptAction, TreeCommand};

use super::keymap::{Context as MapContext, Sequence};
use super::{Focus, Input, Key, Outcome};

impl Input {
    pub(super) fn map_context(&self) -> MapContext {
        if self.focus() != Focus::Editing {
            return MapContext::Minibuffer;
        }
        if self.workspace_tree {
            return MapContext::Tree;
        }
        MapContext::Editor
    }

    pub(super) fn context_binding(&mut self, binding: Binding, key: Key, custom: bool) -> Outcome {
        if binding == Binding::Command(Command::KeyboardQuit) {
            self.cancel();
            return Outcome::Command(Command::KeyboardQuit);
        }
        if self.focus() == Focus::Editing {
            let Binding::Command(command) = binding else {
                unreachable!("workspace maps admit command targets only")
            };
            return Outcome::Command(command);
        }
        // Existing owners retain completion, history, literal search tabs and
        // argument consumption. Inherited keys keep their audited stage limits
        // (for example inventory arrows versus command-history C-p/C-n).
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
        // Decisions accept only their native explicit answers. A single-chord
        // cancel alias may dismiss them, but no mapped accept/edit can consent.
        let cancel = Sequence::from_chords(&[key]).is_some_and(|sequence| {
            self.keymaps.map(MapContext::Minibuffer).lookup(&sequence)
                == Some(Binding::Command(Command::KeyboardQuit))
        });
        if cancel {
            self.cancel();
            return Outcome::Command(Command::KeyboardQuit);
        }
        let outcome = self.focused(key);
        self.consume(outcome)
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
