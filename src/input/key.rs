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
        key.modifier_aliases();
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
        if self.code == KeyCode::Null && self.modifiers.is_empty() {
            self.modifiers = KeyModifiers::CONTROL;
        }
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

    fn modifier_aliases(&mut self) {
        if self.modifiers.contains(KeyModifiers::META) {
            self.modifiers.remove(KeyModifiers::META);
            self.modifiers.insert(KeyModifiers::ALT);
        }
        // Crossterm reports the traditional 0x1f undo byte as C-7 on Unix.
        if self.modifiers == KeyModifiers::CONTROL && self.code == KeyCode::Char('7') {
            self.code = KeyCode::Char('_');
        }
    }
}
