use crossterm::event::{KeyCode, KeyModifiers};

use crate::command::{Command, Motion, Scroll};

use super::Key;

pub(super) fn binding(key: Key) -> Option<Command> {
    if key.modifiers == KeyModifiers::CONTROL {
        return control(key.code);
    }
    if key.modifiers == KeyModifiers::ALT {
        return meta(key.code);
    }
    plain(key.plain()?)
}

fn plain(code: KeyCode) -> Option<Command> {
    match code {
        KeyCode::Enter => Some(Command::Newline),
        KeyCode::Tab => Some(Command::InsertTab),
        KeyCode::Backspace => Some(Command::BackwardDeleteChar),
        KeyCode::Delete => Some(Command::DeleteChar),
        _ => navigation(code),
    }
}

fn navigation(code: KeyCode) -> Option<Command> {
    match code {
        KeyCode::Left => Some(Command::Motion(Motion::BackwardChar)),
        KeyCode::Right => Some(Command::Motion(Motion::ForwardChar)),
        KeyCode::Home => Some(Command::Motion(Motion::LineStart)),
        KeyCode::End => Some(Command::Motion(Motion::LineEnd)),
        KeyCode::Up => Some(Command::Scroll(Scroll::PreviousLine)),
        KeyCode::Down => Some(Command::Scroll(Scroll::NextLine)),
        KeyCode::PageUp => Some(Command::Scroll(Scroll::PageBackward)),
        KeyCode::PageDown => Some(Command::Scroll(Scroll::PageForward)),
        _ => None,
    }
}

fn control(code: KeyCode) -> Option<Command> {
    match code {
        KeyCode::Char('d') => Some(Command::DeleteChar),
        KeyCode::Char('f') => Some(Command::Motion(Motion::ForwardChar)),
        KeyCode::Char('b') => Some(Command::Motion(Motion::BackwardChar)),
        KeyCode::Char('a') => Some(Command::Motion(Motion::LineStart)),
        KeyCode::Char('e') => Some(Command::Motion(Motion::LineEnd)),
        KeyCode::Char('n') => Some(Command::Scroll(Scroll::NextLine)),
        KeyCode::Char('p') => Some(Command::Scroll(Scroll::PreviousLine)),
        KeyCode::Char('v') => Some(Command::Scroll(Scroll::PageForward)),
        KeyCode::Char('l') => Some(Command::Scroll(Scroll::Recenter)),
        _ => None,
    }
}

fn meta(code: KeyCode) -> Option<Command> {
    match code {
        KeyCode::Char('f') => Some(Command::Motion(Motion::ForwardWord)),
        KeyCode::Char('b') => Some(Command::Motion(Motion::BackwardWord)),
        KeyCode::Char('<') => Some(Command::Motion(Motion::BufferStart)),
        KeyCode::Char('>') => Some(Command::Motion(Motion::BufferEnd)),
        KeyCode::Char('v') => Some(Command::Scroll(Scroll::PageBackward)),
        _ => None,
    }
}
