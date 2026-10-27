use crossterm::event::{KeyCode, KeyModifiers};

use crate::command::{
    BufferCommand, Command, History, KillCommand, LinearCommand, Motion, RectangleCommand,
    RegionCommand, Scroll, Split, WindowCommand,
};

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
        KeyCode::Char('s') => Some(Command::SearchForward),
        KeyCode::Char('r') => Some(Command::SearchBackward),
        KeyCode::Char(' ' | '@') => Some(Command::Region(RegionCommand::Set)),
        KeyCode::Null => Some(Command::Region(RegionCommand::Set)),
        KeyCode::Char('k') => Some(Command::Kill(KillCommand::Line)),
        KeyCode::Char('w') => Some(Command::Linear(LinearCommand::Kill)),
        KeyCode::Char('y') => Some(Command::Linear(LinearCommand::Yank)),
        _ => control_edit(code),
    }
}

fn control_edit(code: KeyCode) -> Option<Command> {
    match code {
        KeyCode::Char('/' | '_') => Some(Command::History(History::Undo)),
        _ => control_motion(code),
    }
}

fn control_motion(code: KeyCode) -> Option<Command> {
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
        KeyCode::Char('%') => Some(Command::QueryReplace),
        KeyCode::Char('x') => Some(Command::ExecuteExtended),
        KeyCode::Char('d') => Some(Command::Kill(KillCommand::Word)),
        KeyCode::Backspace => Some(Command::Kill(KillCommand::BackwardWord)),
        KeyCode::Char('w') => Some(Command::Linear(LinearCommand::Copy)),
        KeyCode::Char('y') => Some(Command::Linear(LinearCommand::Pop)),
        _ => meta_motion(code),
    }
}

fn meta_motion(code: KeyCode) -> Option<Command> {
    match code {
        KeyCode::Char('f') => Some(Command::Motion(Motion::ForwardWord)),
        KeyCode::Char('b') => Some(Command::Motion(Motion::BackwardWord)),
        KeyCode::Char('<') => Some(Command::Motion(Motion::BufferStart)),
        KeyCode::Char('>') => Some(Command::Motion(Motion::BufferEnd)),
        KeyCode::Char('v') => Some(Command::Scroll(Scroll::PageBackward)),
        _ => None,
    }
}

pub(super) fn control_x(key: Key) -> Option<Command> {
    if key.modifiers == KeyModifiers::CONTROL {
        return control_suffix(key.code);
    }
    match key.plain()? {
        KeyCode::Char(' ') => Some(Command::Rectangle(RectangleCommand::Mark)),
        KeyCode::Char('b') => Some(Command::Buffer(BufferCommand::Switch)),
        KeyCode::Char('k') => Some(Command::Buffer(BufferCommand::Kill)),
        KeyCode::Char('u') => Some(Command::History(History::Undo)),
        code => window(code),
    }
}

fn control_suffix(code: KeyCode) -> Option<Command> {
    match code {
        KeyCode::Char('x') => Some(Command::Region(RegionCommand::Exchange)),
        KeyCode::Char('s') => Some(Command::SaveBuffer),
        KeyCode::Char('w') => Some(Command::WriteFile),
        KeyCode::Char('f') => Some(Command::FindFile),
        KeyCode::Char('c') => Some(Command::Quit),
        KeyCode::Char('b') => Some(Command::Buffer(BufferCommand::List)),
        _ => None,
    }
}

fn window(code: KeyCode) -> Option<Command> {
    match code {
        KeyCode::Char('2') => Some(Command::Split(Split::Below)),
        KeyCode::Char('3') => Some(Command::Split(Split::Right)),
        KeyCode::Char('0') => Some(Command::Window(WindowCommand::Delete)),
        KeyCode::Char('1') => Some(Command::Window(WindowCommand::Keep)),
        KeyCode::Char('o') => Some(Command::Window(WindowCommand::Other)),
        _ => None,
    }
}
