#![forbid(unsafe_code)]
#[path = "org_io/export.rs"]
mod export;
#[path = "org_io/failures.rs"]
mod failures;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "org_io/state.rs"]
mod state;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fixture::Fixture;
use rev::buffer::Buffers;
use rev::command::{Command, OrgCommand as C};
use rev::config::prepare;
use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};
use rev::session::{Action, Session};
use rev::view::{Size, View};
use std::path::Path;
use std::time::{Duration, Instant, UNIX_EPOCH};

fn clock(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    ZonedOperation::capture(&ZoneSelection::Utc, &ZoneEnvironment::system(), || {
        Ok(UNIX_EPOCH + Duration::from_secs(1709303437))
    })
}
fn later(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    ZonedOperation::capture(&ZoneSelection::Utc, &ZoneEnvironment::system(), || {
        Ok(UNIX_EPOCH + Duration::from_secs(1709307061))
    })
}
fn host(fixture: &Fixture, path: Option<&Path>, persist: bool) -> Session {
    let config = fixture.write(
        "config.toml",
        &format!("org.clock.persist={persist}\norg.time.zone='UTC'"),
    );
    let startup = prepare(&config);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = path
        .map(|p| buffers.visit(p).unwrap())
        .unwrap_or(buffers.initial());
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap()
    .with_org_clock(clock)
}
fn settle(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        if session.remaining_input(Instant::now()).is_none() {
            break;
        }
        assert!(Instant::now() < deadline);
        session.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}
fn action(session: &mut Session, command: C) {
    session.execute(Command::Org(command));
    settle(session);
}
fn argument(session: &mut Session, text: &str) {
    session.handle(Event::Paste(text.into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    settle(session);
}
fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn cancel(session: &mut Session) {
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
}
fn saved(fixture: &Fixture) -> (Session, std::path::PathBuf, std::path::PathBuf) {
    let source = fixture.write("clock-\"-\\-界.org", "\u{feff}* Task 界\r\nbody");
    let state = fixture.0.join("state/org-clock.toml");
    let mut session = host(fixture, Some(&source), true);
    assert_eq!(session.restore_clock_state(&state).unwrap(), Action::Wait);
    action(&mut session, C::ClockIn);
    assert!(session.echo().contains("started"), "{}", session.echo());
    session.execute(Command::SaveBuffer);
    assert!(session.echo().contains("Saved"), "{}", session.echo());
    (session, source, state)
}

#[path = "org_io/agenda_sources.rs"]
mod agenda_sources;

#[path = "org_io/capture.rs"]
mod capture;

#[path = "org_io/capture_bounds.rs"]
mod capture_bounds;

#[path = "org_io/refile.rs"]
mod refile;
#[path = "org_io/refile_failures.rs"]
mod refile_failures;

#[path = "org_io/archive.rs"]
mod archive;

#[path = "org_io/links.rs"]
mod links;

#[path = "org_io/link_viewer.rs"]
mod link_viewer;

#[path = "org_io/link_edges.rs"]
mod link_edges;

#[path = "org_io/attachments.rs"]
mod attachments;

#[path = "org_io/attachment_edges.rs"]
mod attachment_edges;

#[path = "org_io/table_interchange.rs"]
mod table_interchange;

#[path = "org_io/text_exports.rs"]
mod text_exports;

#[path = "org_io/html_exports.rs"]
mod html_exports;

#[path = "org_io/tex_exports.rs"]
mod tex_exports;

#[path = "org_io/tex_jobs.rs"]
mod tex_jobs;

#[path = "org_io/odt_exports.rs"]
mod odt_exports;

#[path = "org_io/manual_exports.rs"]
mod manual_exports;

#[path = "org_io/calendar_exports.rs"]
mod calendar_exports;

#[path = "org_io/publishing.rs"]
mod publishing;

#[path = "org_io/tangling.rs"]
mod tangling;
