use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// A normalized key; releases and inappropriate repeats never reach routing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl Key {
    pub(super) fn normalize(event: KeyEvent) -> Option<Self> {
        if event.kind == KeyEventKind::Release {
            return None;
        }
        let mut key = Self {
            code: event.code,
            modifiers: event.modifiers,
        };
        key.character();
        if event.kind == KeyEventKind::Repeat && !key.repeatable() {
            return None;
        }
        Some(key)
    }

    pub(super) fn control(self, character: char) -> bool {
        self.modifiers == KeyModifiers::CONTROL && self.code == KeyCode::Char(character)
    }

    pub(super) fn plain(self) -> Option<KeyCode> {
        if self.modifiers.is_empty() {
            return Some(self.code);
        }
        None
    }

    fn character(&mut self) {
        if let KeyCode::Char(character) = self.code {
            self.modifiers.remove(KeyModifiers::SHIFT);
            if self.modifiers.contains(KeyModifiers::CONTROL) && character.is_ascii_alphabetic() {
                self.code = KeyCode::Char(character.to_ascii_lowercase());
            }
        }
    }

    fn repeatable(self) -> bool {
        super::bindings::binding(self).is_some() || matches!(self.plain(), Some(KeyCode::Char(_)))
    }
}
