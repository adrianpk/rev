use crossterm::event::KeyCode;
use unicode_segmentation::UnicodeSegmentation;

use super::{Error, Key, Outcome};

/// Compiled default; each prompt enforces its admitted scalar limit.
pub const LIMIT: usize = 4096;

pub(super) const LIMIT_MESSAGE: &str = "Prompt input limit exceeded";

/// Single-line argument input; cursor is a byte offset at a grapheme boundary.
/// The bounded string belongs to the prompt and never aliases document text.
#[derive(Debug)]
pub struct Prompt {
    label: String,
    text: String,
    cursor: usize,
    limit: usize,
}

impl Prompt {
    pub(super) fn new(label: &str, initial: &str, limit: usize) -> Result<Self, Error> {
        if label.len() > 1024 || initial.chars().count() > limit {
            return Err(Error::Limit);
        }
        if label.chars().any(char::is_control) || initial.chars().any(char::is_control) {
            return Err(Error::Control);
        }
        Ok(Self {
            label: label.into(),
            limit,
            text: initial.into(),
            cursor: initial.len(),
        })
    }

    pub(super) fn replace(&mut self, text: &str) -> Result<(), Error> {
        *self = Self::new(&self.label, text, self.limit)?;
        Ok(())
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub(super) fn set_label(&mut self, label: &'static str) {
        self.label = label.into();
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub(super) fn handle(&mut self, key: Key) -> Outcome {
        let Some(code) = key.plain().or_else(|| control_code(key)) else {
            return Outcome::Message("Unbound prompt key");
        };
        match code {
            KeyCode::Char(character) => self.insert(character),
            KeyCode::Enter => Outcome::Argument(std::mem::take(&mut self.text)),
            KeyCode::Tab => Outcome::Complete,
            _ => self.edit(code),
        }
    }

    pub(super) fn paste(&mut self, text: &str) -> Outcome {
        if text.chars().any(char::is_control) {
            return Outcome::Message("Control character rejected");
        }
        self.literal_paste(text)
    }

    pub(super) fn literal_paste(&mut self, text: &str) -> Outcome {
        let count = self
            .text
            .chars()
            .count()
            .saturating_add(text.chars().count());
        if count > self.limit {
            return Outcome::Message(LIMIT_MESSAGE);
        }
        self.text.insert_str(self.cursor, text);
        self.cursor += text.len();
        self.repair();
        Outcome::Changed
    }

    fn edit(&mut self, code: KeyCode) -> Outcome {
        match code {
            KeyCode::Backspace => self.erase(self.previous()..self.cursor),
            KeyCode::Delete => self.erase(self.cursor..self.next()),
            KeyCode::Left => self.cursor = self.previous(),
            KeyCode::Right => self.cursor = self.next(),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.text.len(),
            _ => return Outcome::Message("Unbound prompt key"),
        }
        Outcome::Changed
    }

    fn insert(&mut self, character: char) -> Outcome {
        if character.is_control() {
            return Outcome::Message("Control character rejected");
        }
        if self.text.chars().count() == self.limit {
            return Outcome::Message(LIMIT_MESSAGE);
        }
        self.text.insert(self.cursor, character);
        self.cursor += character.len_utf8();
        self.repair();
        Outcome::Changed
    }

    fn erase(&mut self, range: std::ops::Range<usize>) {
        self.cursor = range.start;
        self.text.replace_range(range, "");
        self.repair();
    }

    fn previous(&self) -> usize {
        self.text
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .take_while(|index| *index < self.cursor)
            .last()
            .unwrap_or(0)
    }

    fn next(&self) -> usize {
        self.text
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .find(|index| *index > self.cursor)
            .unwrap_or(self.text.len())
    }

    fn repair(&mut self) {
        self.cursor = self
            .text
            .grapheme_indices(true)
            .map(|(index, _)| index)
            .find(|index| *index >= self.cursor)
            .unwrap_or(self.text.len());
    }
}

fn control_code(key: Key) -> Option<KeyCode> {
    use crossterm::event::KeyModifiers;
    if key.modifiers != KeyModifiers::CONTROL {
        return None;
    }
    match key.code {
        KeyCode::Char('a') => Some(KeyCode::Home),
        KeyCode::Char('e') => Some(KeyCode::End),
        KeyCode::Char('b') => Some(KeyCode::Left),
        KeyCode::Char('f') => Some(KeyCode::Right),
        KeyCode::Char('h') => Some(KeyCode::Backspace),
        KeyCode::Char('d') => Some(KeyCode::Delete),
        _ => None,
    }
}
