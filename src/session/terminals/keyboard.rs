use alacritty_terminal::term::TermMode;
use crossterm::event::{KeyCode, KeyModifiers};

use crate::input::Key;

/// Classic VT input only. Extended keyboard negotiation stays disabled in the
/// engine, so the child cannot request a protocol the native adapter lacks.
pub(super) fn encode(key: Key, mode: TermMode, keypad: bool) -> Option<Vec<u8>> {
    if keypad && mode.contains(TermMode::APP_KEYPAD) {
        return application_keypad(key);
    }
    if let KeyCode::Char(c) = key.code {
        return character(c, key.modifiers);
    }
    if let Some(bytes) = simple(key, mode) {
        return Some(bytes);
    }
    named(key, mode)
}

fn character(c: char, modifiers: KeyModifiers) -> Option<Vec<u8>> {
    let bytes = if modifiers.contains(KeyModifiers::CONTROL) {
        let byte = match c {
            ' '..='?' | '@'..='_' | 'a'..='z' => (c as u8) & 0x1f,
            _ => return None,
        };
        vec![byte]
    } else {
        c.to_string().into_bytes()
    };
    Some(meta(bytes, modifiers))
}

fn meta(mut bytes: Vec<u8>, modifiers: KeyModifiers) -> Vec<u8> {
    if modifiers.contains(KeyModifiers::ALT) {
        bytes.insert(0, 0x1b);
    }
    bytes
}

fn simple(key: Key, mode: TermMode) -> Option<Vec<u8>> {
    let bytes: &[u8] = match key.code {
        KeyCode::Enter if mode.contains(TermMode::LINE_FEED_NEW_LINE) => b"\r\n",
        KeyCode::Enter => b"\r",
        KeyCode::Tab => b"\t",
        KeyCode::Backspace if key.modifiers.contains(KeyModifiers::CONTROL) => b"\x08",
        KeyCode::Backspace => b"\x7f",
        KeyCode::Esc => b"\x1b",
        _ => return None,
    };
    Some(meta(bytes.to_vec(), key.modifiers))
}

fn modifier(modifiers: KeyModifiers) -> u8 {
    1 + u8::from(modifiers.contains(KeyModifiers::SHIFT))
        + 2 * u8::from(modifiers.contains(KeyModifiers::ALT))
        + 4 * u8::from(modifiers.contains(KeyModifiers::CONTROL))
}

fn named(key: Key, mode: TermMode) -> Option<Vec<u8>> {
    let modifier = modifier(key.modifiers);
    if let Some(end) = cursor(key.code) {
        let bytes = if modifier > 1 {
            format!("\x1b[1;{modifier}{end}")
        } else if mode.contains(TermMode::APP_CURSOR) {
            format!("\x1bO{end}")
        } else {
            format!("\x1b[{end}")
        };
        return Some(bytes.into_bytes());
    }
    if key.code == KeyCode::BackTab {
        return Some(b"\x1b[Z".to_vec());
    }
    function(key.code, modifier)
}

fn cursor(code: KeyCode) -> Option<char> {
    match code {
        KeyCode::Up => Some('A'),
        KeyCode::Down => Some('B'),
        KeyCode::Right => Some('C'),
        KeyCode::Left => Some('D'),
        KeyCode::Home => Some('H'),
        KeyCode::End => Some('F'),
        KeyCode::KeypadBegin => Some('E'),
        _ => None,
    }
}

fn function(code: KeyCode, modifier: u8) -> Option<Vec<u8>> {
    if let KeyCode::F(number @ 1..=4) = code {
        let end = char::from(b'P' + number - 1);
        return Some(if modifier == 1 {
            format!("\x1bO{end}").into_bytes()
        } else {
            format!("\x1b[1;{modifier}{end}").into_bytes()
        });
    }
    let number = tilde(code)?;
    Some(if modifier == 1 {
        format!("\x1b[{number}~").into_bytes()
    } else {
        format!("\x1b[{number};{modifier}~").into_bytes()
    })
}

fn tilde(code: KeyCode) -> Option<u8> {
    match code {
        KeyCode::Insert => Some(2),
        KeyCode::Delete => Some(3),
        KeyCode::PageUp => Some(5),
        KeyCode::PageDown => Some(6),
        KeyCode::F(n @ 5..=20) => Some(
            [
                15, 17, 18, 19, 20, 21, 23, 24, 25, 26, 28, 29, 31, 32, 33, 34,
            ][usize::from(n - 5)],
        ),
        _ => None,
    }
}

fn application_keypad(key: Key) -> Option<Vec<u8>> {
    let end = match key.code {
        KeyCode::Char(c @ '0'..='9') => char::from(b'p' + c as u8 - b'0'),
        KeyCode::Char(c) => {
            let position = "*+,-./=".find(c)?;
            char::from(b"jklmnoX"[position])
        }
        KeyCode::Enter => 'M',
        _ => return named(key, TermMode::APP_CURSOR),
    };
    Some(meta(format!("\x1bO{end}").into_bytes(), key.modifiers))
}
