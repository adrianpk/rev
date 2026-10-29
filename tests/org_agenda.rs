#![forbid(unsafe_code)]
#[path = "org_agenda/contracts.rs"]
mod contracts;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "org_agenda/reports.rs"]
mod reports;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fixture::Fixture;
use rev::{
    buffer::Buffers,
    command::{Command, OrgCommand},
    config::prepare,
    session::Session,
    view::{Size, View},
};
use std::time::Instant;
fn editor(config: &str, name: &str, source: &str) -> (Fixture, Session) {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&fixture.write(name, source)).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap();
    (fixture, session)
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn settle(session: &mut Session) {
    let until = Instant::now() + std::time::Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        if session.remaining_input(Instant::now()).is_none() {
            return;
        }
        assert!(Instant::now() < until, "owned Org work did not finish");
        session.expire_input(Instant::now() + std::time::Duration::from_millis(10));
        std::thread::yield_now();
    }
}

fn action(session: &mut Session, command: OrgCommand) {
    session.execute(Command::Org(command));
    settle(session);
}
fn argument(session: &mut Session, value: &str) {
    session.handle(Event::Paste(value.into()));
    session.handle(key(KeyCode::Enter));
}
fn goto(session: &mut Session, line: usize) {
    session.execute(Command::GotoLine);
    argument(session, &line.to_string());
}
fn control(character: char) -> Event {
    Event::Key(KeyEvent::new(
        KeyCode::Char(character),
        KeyModifiers::CONTROL,
    ))
}
#[path = "org_agenda/habit_views.rs"]
mod habit_views;
#[path = "org_agenda/habits.rs"]
mod habits;

#[path = "org_agenda/sources.rs"]
mod sources;

#[path = "org_agenda/calendar_rows.rs"]
mod calendar_rows;

#[path = "org_agenda/calendar_native.rs"]
mod calendar_native;

#[path = "org_agenda/bulk_actions.rs"]
mod bulk_actions;
#[path = "org_agenda/calendar_views.rs"]
mod calendar_views;
#[path = "org_agenda/custom_native.rs"]
mod custom_native;
#[path = "org_agenda/custom_views.rs"]
mod custom_views;
#[path = "org_agenda/query_language.rs"]
mod query_language;
#[path = "org_agenda/query_native.rs"]
mod query_native;
#[path = "org_agenda/query_results.rs"]
mod query_results;
#[path = "org_agenda/source_actions.rs"]
mod source_actions;

#[path = "org_agenda/action_lifecycle.rs"]
mod action_lifecycle;
#[path = "org_agenda/capture_expansion.rs"]
mod capture_expansion;
#[path = "org_agenda/capture_publication.rs"]
mod capture_publication;
#[path = "org_agenda/capture_shapes.rs"]
mod capture_shapes;

#[path = "org_agenda/columns.rs"]
mod columns;
