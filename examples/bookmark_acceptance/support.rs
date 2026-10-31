use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{BookmarkCommand, Command};
use rev::config::Snapshot;
use rev::session::{Action, Session};
use rev::view::{Size, View};

pub(super) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(super) const SIZE: Size = Size {
    width: 72,
    height: 24,
};

pub(super) fn key(session: &mut Session, code: KeyCode) {
    session.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}

pub(super) fn answer(session: &mut Session, text: &str) {
    session.handle(Event::Paste(text.into()));
    key(session, KeyCode::Enter);
}

pub(super) fn bookmark(session: &mut Session, command: BookmarkCommand, text: &str) {
    session.execute(Command::Bookmark(command));
    answer(session, text);
}

pub(super) fn goto(session: &mut Session, line: usize) {
    session.execute(Command::GotoLine);
    answer(session, &line.to_string());
}

pub(super) fn source(session: &Session) -> Vec<u8> {
    let mut bytes = Vec::new();
    rev::file::write(session.buffer().unwrap(), &mut bytes).unwrap();
    bytes
}

pub(super) fn configured(settings: &Snapshot, path: Option<&Path>) -> Result<Session> {
    let mut buffers = Buffers::with_settings(settings.clone())?;
    let id = match path {
        Some(path) => buffers.visit(path)?,
        None => buffers.initial(),
    };
    let view = View::with_settings(buffers.get(id)?, settings);
    Ok(Session::new(buffers, view, SIZE)?)
}

/// Finish only session-owned work, retaining the existing finite native deadline.
pub(super) fn settle(session: &mut Session) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        session.frame()?;
        let now = Instant::now();
        if session.remaining_input(now).is_none() {
            return Ok(());
        }
        assert!(now < deadline, "native bookmark work did not settle");
        session.expire_input(now);
        std::thread::yield_now();
    }
}

pub(super) fn idle(session: &mut Session) {
    assert_eq!(session.remaining_input(Instant::now()), None);
    assert_eq!(
        session.expire_input(Instant::now() + Duration::from_secs(10)),
        Action::Wait
    );
}

pub(super) fn markers(session: &mut Session) -> Result<usize> {
    Ok(session
        .frame()?
        .rows()
        .iter()
        .map(|row| row.text().matches('●').count())
        .sum())
}
