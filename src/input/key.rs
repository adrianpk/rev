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
        key.terminal_aliases();
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

    fn modifier_aliases(&mut self) {
        if self.modifiers.contains(KeyModifiers::META) {
            self.modifiers.remove(KeyModifiers::META);
            self.modifiers.insert(KeyModifiers::ALT);
        }
    }
}

impl Key {
    fn shift_aliases(&mut self) {
        if self.modifiers == KeyModifiers::SHIFT && self.code == KeyCode::Tab {
            self.code = KeyCode::BackTab;
            self.modifiers = KeyModifiers::NONE;
        }
        if self.code == KeyCode::BackTab {
            self.modifiers.remove(KeyModifiers::SHIFT);
        }
    }

    fn terminal_aliases(&mut self) {
        self.shift_aliases();
        if !self.modifiers.contains(KeyModifiers::CONTROL) {
            return;
        }
        // Traditional terminal reports collapse these aliases, including C-7
        // for the Unix 0x1f undo byte. Alt remains present on Meta aliases.
        match self.code {
            KeyCode::Char('i') => self.unmodified(KeyCode::Tab),
            KeyCode::Char('j' | 'm') => self.unmodified(KeyCode::Enter),
            KeyCode::Char('h' | '?') => self.unmodified(KeyCode::Backspace),
            KeyCode::Char('[') => self.unmodified(KeyCode::Esc),
            _ => self.control_identity(),
        }
    }

    fn control_identity(&mut self) {
        let aliases = [
            (KeyCode::Null, KeyCode::Char(' '), false),
            (KeyCode::Char('@'), KeyCode::Char(' '), false),
            (KeyCode::Char('2'), KeyCode::Char(' '), false),
            (KeyCode::Char('3'), KeyCode::Esc, true),
            (KeyCode::Char('4'), KeyCode::Char('\\'), false),
            (KeyCode::Char('5'), KeyCode::Char(']'), false),
            (KeyCode::Char('6'), KeyCode::Char('^'), false),
            (KeyCode::Char('/'), KeyCode::Char('_'), false),
            (KeyCode::Char('7'), KeyCode::Char('_'), false),
            (KeyCode::Char('8'), KeyCode::Backspace, true),
        ];
        if let Some((_, code, plain)) = aliases
            .into_iter()
            .find(|(alias, _, _)| *alias == self.code)
        {
            self.code = code;
            if plain {
                self.modifiers.remove(KeyModifiers::CONTROL);
            }
        }
    }

    fn unmodified(&mut self, code: KeyCode) {
        self.code = code;
        self.modifiers.remove(KeyModifiers::CONTROL);
    }
}
