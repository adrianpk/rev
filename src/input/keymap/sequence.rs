use std::{cmp::Ordering, fmt};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::input::Key;

/// At most eight normalized chords. Escape followed by a plain chord is the
/// same Meta chord as reported Alt/Meta; it cannot form a distinct binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sequence(Vec<Key>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxError(pub &'static str);

impl fmt::Display for SyntaxError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str(self.0)
    }
}

impl std::error::Error for SyntaxError {}

impl Sequence {
    pub const MAX_CHORDS: usize = 8;

    pub fn parse(text: &str) -> Result<Self, SyntaxError> {
        let mut keys = Vec::new();
        let mut tokens = text.split_ascii_whitespace();
        while let Some(token) = tokens.next() {
            let mut key = chord(token)?;
            if key.plain() == Some(KeyCode::Esc) {
                key = escaped(tokens.next())?;
            }
            keys.push(key);
            if keys.len() > Self::MAX_CHORDS {
                return Err(SyntaxError("key sequence exceeds eight normalized chords"));
            }
        }
        if keys.is_empty() {
            return Err(SyntaxError("expected a nonempty key sequence"));
        }
        Ok(Self(keys))
    }

    /// Only normalized runtime keys enter this constructor. Unsupported native
    /// key codes have no configured spelling and never reach sequence ordering.
    pub(crate) fn from_chords(chords: &[Key]) -> Option<Self> {
        let admitted = !chords.is_empty()
            && chords.len() <= Self::MAX_CHORDS
            && chords.iter().all(|key| supported(key.code));
        admitted.then(|| Self(chords.to_vec()))
    }

    pub fn chords(&self) -> &[Key] {
        &self.0
    }
}

fn escaped(token: Option<&str>) -> Result<Key, SyntaxError> {
    let mut key = chord(token.ok_or(SyntaxError("Escape requires a following plain Meta chord"))?)?;
    if !key.modifiers.is_empty() || key.code == KeyCode::Esc {
        return Err(SyntaxError(
            "Escape Meta prefix requires a plain non-Escape chord",
        ));
    }
    key.modifiers = KeyModifiers::ALT;
    Ok(key)
}

fn chord(token: &str) -> Result<Key, SyntaxError> {
    let (modifiers, name) = modifiers(token)?;
    let code = code(name)?;
    if modifiers.contains(KeyModifiers::SHIFT) && matches!(code, KeyCode::Char(_)) {
        return Err(SyntaxError(
            "use the literal uppercase or shifted character, not S-character",
        ));
    }
    Key::normalize(KeyEvent::new(code, modifiers)).ok_or(SyntaxError("unsupported key event"))
}

fn modifiers(mut text: &str) -> Result<(KeyModifiers, &str), SyntaxError> {
    let mut modifiers = KeyModifiers::NONE;
    while let Some((prefix, rest)) = text.split_once('-') {
        let modifier = match prefix {
            "C" => KeyModifiers::CONTROL,
            "M" | "A" => KeyModifiers::ALT,
            "S" => KeyModifiers::SHIFT,
            _ => break,
        };
        if modifiers.contains(modifier) {
            return Err(SyntaxError("duplicate key modifier"));
        }
        modifiers.insert(modifier);
        text = rest;
    }
    Ok((modifiers, text))
}

fn code(name: &str) -> Result<KeyCode, SyntaxError> {
    let named = named_code(name).or_else(|| navigation(name));
    if let Some(code) = named {
        return Ok(code);
    }
    let mut characters = name.chars();
    let character = characters.next().ok_or(SyntaxError("missing chord key"))?;
    if characters.next().is_some() || character.is_control() || character.is_whitespace() {
        return Err(SyntaxError(
            "expected a printable scalar or supported named key",
        ));
    }
    Ok(KeyCode::Char(character))
}

fn navigation(name: &str) -> Option<KeyCode> {
    match name.to_ascii_lowercase().as_str() {
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" => Some(KeyCode::PageUp),
        "pagedown" => Some(KeyCode::PageDown),
        "backtab" => Some(KeyCode::BackTab),
        _ => None,
    }
}

fn named_code(name: &str) -> Option<KeyCode> {
    let name = name.to_ascii_lowercase();
    [
        ("ret", KeyCode::Enter),
        ("return", KeyCode::Enter),
        ("enter", KeyCode::Enter),
        ("tab", KeyCode::Tab),
        ("spc", KeyCode::Char(' ')),
        ("space", KeyCode::Char(' ')),
        ("del", KeyCode::Backspace),
        ("backspace", KeyCode::Backspace),
        ("delete", KeyCode::Delete),
        ("esc", KeyCode::Esc),
        ("escape", KeyCode::Esc),
        ("null", KeyCode::Null),
    ]
    .into_iter()
    .find(|(alias, _)| *alias == name)
    .map(|(_, code)| code)
}

impl PartialOrd for Sequence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Sequence {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.iter().map(order).cmp(other.0.iter().map(order))
    }
}

fn order(key: &Key) -> (u8, (u8, u32)) {
    (key.modifiers.bits(), code_order(key.code))
}

fn code_order(code: KeyCode) -> (u8, u32) {
    match code {
        KeyCode::Char(character) => (0, u32::from(character)),
        KeyCode::Enter => (1, 0),
        KeyCode::Tab => (2, 0),
        KeyCode::BackTab => (3, 0),
        KeyCode::Backspace => (4, 0),
        KeyCode::Delete => (5, 0),
        KeyCode::Esc => (6, 0),
        _ => (7, navigation_order(code)),
    }
}

fn navigation_order(code: KeyCode) -> u32 {
    match code {
        KeyCode::Left => 0,
        KeyCode::Right => 1,
        KeyCode::Up => 2,
        KeyCode::Down => 3,
        KeyCode::Home => 4,
        KeyCode::End => 5,
        KeyCode::PageUp => 6,
        KeyCode::PageDown => 7,
        _ => unreachable!("sequences contain only parsed named keys and scalars"),
    }
}

fn supported(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::Char(_)
            | KeyCode::Enter
            | KeyCode::Tab
            | KeyCode::BackTab
            | KeyCode::Backspace
            | KeyCode::Delete
            | KeyCode::Esc
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::PageUp
            | KeyCode::PageDown
    )
}

impl fmt::Display for Sequence {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, key) in self.0.iter().enumerate() {
            if index > 0 {
                output.write_str(" ")?;
            }
            write_modifiers(output, key.modifiers)?;
            output.write_str(&key_name(key.code))?;
        }
        Ok(())
    }
}

fn write_modifiers(output: &mut fmt::Formatter<'_>, modifiers: KeyModifiers) -> fmt::Result {
    for (modifier, name) in [
        (KeyModifiers::CONTROL, "C-"),
        (KeyModifiers::ALT, "M-"),
        (KeyModifiers::SHIFT, "S-"),
    ] {
        if modifiers.contains(modifier) {
            output.write_str(name)?;
        }
    }
    Ok(())
}

fn key_name(code: KeyCode) -> String {
    if let KeyCode::Char(character) = code {
        return if character == ' ' {
            "SPC".into()
        } else {
            character.to_string()
        };
    }
    [
        (KeyCode::Enter, "RET"),
        (KeyCode::Tab, "TAB"),
        (KeyCode::BackTab, "BackTab"),
        (KeyCode::Backspace, "Backspace"),
        (KeyCode::Delete, "Delete"),
        (KeyCode::Esc, "ESC"),
        (KeyCode::Left, "Left"),
        (KeyCode::Right, "Right"),
        (KeyCode::Up, "Up"),
        (KeyCode::Down, "Down"),
        (KeyCode::Home, "Home"),
        (KeyCode::End, "End"),
        (KeyCode::PageUp, "PageUp"),
        (KeyCode::PageDown, "PageDown"),
    ]
    .into_iter()
    .find(|(key, _)| *key == code)
    .expect("sequences contain only admitted keys")
    .1
    .into()
}
