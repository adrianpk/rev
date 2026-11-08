#![forbid(unsafe_code)]

#[path = "configuration/fixture.rs"]
mod fixture;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, History, MacroCommand};
use rev::display::Size;
use rev::macros::{Status, StopReason};
use rev::session::Session;
use rev::view::View;
use std::time::{Duration, Instant};

fn editor(config: &str, source: &str) -> Session {
    let fixture = fixture::Fixture::new();
    let prepared = rev::config::prepare(&fixture.write("config.toml", config));
    assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
    let mut buffers = Buffers::with_settings(prepared.snapshot).unwrap();
    let id = buffers.create(source).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 16,
        },
    )
    .unwrap()
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}
fn character(value: char) -> Event {
    key(KeyCode::Char(value))
}
fn control(value: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), KeyModifiers::CONTROL))
}
fn meta(value: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), KeyModifiers::ALT))
}
fn feed(session: &mut Session, events: impl IntoIterator<Item = Event>) {
    for event in events {
        session.handle(event);
    }
}
fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn settle(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while session
        .remaining_input(Instant::now())
        .is_some_and(|wait| wait == Duration::ZERO)
    {
        assert!(Instant::now() < deadline, "native work did not settle");
        session.expire_input(Instant::now());
    }
}
fn record(session: &mut Session, events: impl IntoIterator<Item = Event>) {
    session.handle(key(KeyCode::F(3)));
    assert_eq!(session.macro_status(), Status::Recording);
    assert!(session.frame().unwrap().rows().iter().any(|row| matches!(
        row.band(),
        rev::display::Band::Status | rev::display::Band::Mixed
    ) && row.text().contains("Def")));
    feed(session, events);
    session.handle(key(KeyCode::F(4)));
    assert_eq!(session.macro_status(), Status::Idle);
}

#[path = "macros/recording.rs"]
mod recording;
#[path = "macros/runner.rs"]
mod runner;

#[path = "macros/capture.rs"]
mod capture;

#[path = "macros/counters.rs"]
mod counters;
#[path = "macros/ring.rs"]
mod ring;

#[path = "macros/names.rs"]
mod names;
#[path = "macros/queries.rs"]
mod queries;
#[path = "macros/regions.rs"]
mod regions;

#[path = "macros/library.rs"]
mod library;

#[path = "macros/definition_editor.rs"]
mod definition_editor;
#[path = "macros/lossage.rs"]
mod lossage;
#[path = "macros/menu.rs"]
mod menu;

#[path = "macros/step_edit.rs"]
mod step_edit;

#[path = "macros/pty.rs"]
mod pty;

#[path = "macros/acceptance.rs"]
mod acceptance;
